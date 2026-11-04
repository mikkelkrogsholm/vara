use crate::{Cancellation, Result};
use parking_lot::Mutex;
use std::path::Path;
use voxrt::{
    longform::LongForm,
    ternary_linear::Precision,
    transcriber::{Device, Transcriber},
};

#[derive(Default)]
pub struct Parakeet {
    runtime: Mutex<Option<(Transcriber, rayon::ThreadPool)>>,
}
impl Parakeet {
    pub fn transcript(
        &self,
        dir: &Path,
        audio: &Path,
        cancel: &Cancellation,
        progress: &(dyn Fn(f64) + Sync),
    ) -> Result<vara_core::Transcript> {
        use vara_core::{TimedWord, Timing, Transcript, TranscriptSegment};
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Reuse the same runtime, but feed a bounded streaming reader for long files.
            // The short initialization request is not an inference call.
            let mut guard = self.runtime.lock();
            cancel.check()?;
            if guard.is_none() {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(8)
                    .build()
                    .map_err(|_| vara_core::UserMessage::new("speech.start").wire())?;
                let model = pool
                    .install(|| Transcriber::load_on(dir, Precision::Int8, Device::Auto))
                    .map_err(|_| vara_core::UserMessage::new("parakeet.model_missing").wire())?;
                *guard = Some((model, pool));
            }
            let (model, pool) = guard.as_ref().unwrap();
            pool.install(|| {
                let mut stream = LongForm::new(model);
                let mut reader = hound::WavReader::open(audio)
                    .map_err(|_| vara_core::UserMessage::new("audio.work_open").wire())?;
                let mut input = reader.samples::<i16>();

                loop {
                    cancel.check()?;
                    let chunk = input
                        .by_ref()
                        .take(16000 * 10)
                        .map(|v| v.map(|v| v as f32 / 32768.))
                        .collect::<std::result::Result<Vec<_>, _>>()
                        .map_err(|_| vara_core::UserMessage::new("media.work_corrupt").wire())?;
                    if chunk.is_empty() {
                        break;
                    }
                    stream.push(&chunk);
                    progress(stream.segments.last().map(|s| s.end).unwrap_or(0.));
                }
                cancel.check()?;
                stream.flush();
                cancel.check()?;
                let text = stream.text();
                if text.trim().is_empty() {
                    return Err(vara_core::UserMessage::new("speech.file_empty").wire());
                }
                let segments = stream
                    .segments
                    .iter()
                    .enumerate()
                    .map(|(i, s)| TranscriptSegment {
                        id: format!("s{i}"),
                        text: s.text.clone(),
                        start: Some(s.start),
                        end: Some(s.end),
                        speaker: None,
                        words: s
                            .words
                            .iter()
                            .enumerate()
                            .map(|(j, w)| TimedWord {
                                id: format!("s{i}w{j}"),
                                text: w.word.clone(),
                                start: w.start,
                                end: w.end,
                                speaker: None,
                            })
                            .collect(),
                    })
                    .collect::<Vec<_>>();
                let timing = if segments.iter().any(|s| !s.words.is_empty()) {
                    Timing::Word
                } else {
                    Timing::Segment
                };
                Ok(Transcript {
                    text,
                    segments,
                    timing,
                    timing_source: "asr".into(),
                })
            })
        }));
        match result {
            Ok(r) => r,
            Err(_) => {
                *self.runtime.lock() = None;
                Err(vara_core::UserMessage::new("speech.file_interrupted").wire())
            }
        }
    }
    pub fn speech(&self, dir: &Path, audio: &Path, cancel: &Cancellation) -> Result<String> {
        // A backend panic must become an ordinary failed recognition, retaining retry audio.
        // The host's durable generation gate still owns cancellation/save/delivery.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.recognize(dir, audio, cancel)
        })) {
            Ok(result) => result,
            Err(_) => {
                *self.runtime.lock() = None;
                Err(vara_core::UserMessage::new("speech.recording_interrupted").wire())
            }
        }
    }
    fn recognize(&self, dir: &Path, audio: &Path, cancel: &Cancellation) -> Result<String> {
        cancel.check()?;
        let (samples, _) = crate::local::read_audio(audio)?;
        if samples.len() < 1600 {
            return Err(vara_core::UserMessage::new("audio.too_short").wire());
        }
        let mut guard = self.runtime.lock();
        cancel.check()?;
        if guard.is_none() {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(8)
                .build()
                .map_err(|_| vara_core::UserMessage::new("speech.threads").wire())?;
            // Explicit backend selection ignores external VOXRT_DEVICE overrides.
            // Auto uses Metal on supported Apple builds and CPU elsewhere.
            let model = pool
                .install(|| Transcriber::load_on(dir, Precision::Int8, Device::Auto))
                .map_err(|_| vara_core::UserMessage::new("parakeet.load").wire())?;
            *guard = Some((model, pool));
        }
        cancel.check()?;
        let (model, pool) = guard.as_ref().unwrap();
        pool.install(|| {
            let mut stream = LongForm::new(model);
            for chunk in samples.chunks(10 * 16000) {
                cancel.check()?;
                stream.push(chunk);
            }
            cancel.check()?;
            stream.flush();
            cancel.check()?;
            Ok(stream.text())
        })
    }
}
