use crate::{Cancellation, Result};
use candle_core::{Device, Tensor, quantized::gguf_file};
use candle_transformers::{generation::LogitsProcessor, models::quantized_qwen3::ModelWeights};
use parking_lot::Mutex;
use std::{fs::File, path::Path};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperVadContext,
    WhisperVadContextParams, WhisperVadParams,
};
struct SpeechRuntime {
    whisper: WhisperContext,
    vad: WhisperVadContext,
}
#[derive(Default)]
pub struct Local {
    speech: Mutex<Option<SpeechRuntime>>,
    text: Mutex<Option<(ModelWeights, tokenizers::Tokenizer)>>,
}
impl Local {
    pub fn speech(
        &self,
        model: &Path,
        path: &Path,
        language: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<String> {
        Ok(self.transcribe(model, path, language, cancel, false)?.text)
    }
    pub fn transcript(
        &self,
        model: &Path,
        path: &Path,
        language: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<vara_core::Transcript> {
        self.transcribe(model, path, language, cancel, true)
    }
    fn transcribe(
        &self,
        model: &Path,
        path: &Path,
        language: Option<&str>,
        cancel: &Cancellation,
        timed: bool,
    ) -> Result<vara_core::Transcript> {
        let (samples, _) = read_audio(path)?;
        if samples.len() < 1600 {
            return Err(vara_core::UserMessage::new("audio.too_short").wire());
        }
        let mut guard = self.speech.lock();
        cancel.check()?;
        if guard.is_none() {
            let mut params = WhisperContextParameters::default();
            params.use_gpu(false);
            let directory = model
                .parent()
                .ok_or_else(|| vara_core::UserMessage::new("whisper.load").wire())?;
            // Verify both managed weights once per cached runtime. Inference never downloads.
            for artifact in crate::artifacts("local.whisper").unwrap() {
                if !crate::verified(&directory.join(artifact.name), artifact)? {
                    return Err(vara_core::UserMessage::new("whisper.load")
                        .caused_by(vara_core::UserMessage::new("model.verification"))
                        .wire());
                }
                cancel.check()?;
            }
            let whisper = WhisperContext::new_with_params(
                model
                    .to_str()
                    .ok_or(vara_core::UserMessage::new("model.invalid_path").wire())?,
                params,
            )
            .map_err(|e| {
                vara_core::UserMessage::new("whisper.load")
                    .caused_by(crate::backend_error::whisper(&e))
                    .wire()
            })?;
            let mut vad_params = WhisperVadContextParams::default();
            vad_params.set_use_gpu(false);
            vad_params.set_n_threads(8);
            let vad = WhisperVadContext::new(
                directory
                    .join("ggml-silero-v5.1.2.bin")
                    .to_str()
                    .ok_or_else(|| vara_core::UserMessage::new("model.invalid_path").wire())?,
                vad_params,
            )
            .map_err(|e| {
                vara_core::UserMessage::new("whisper.load")
                    .caused_by(crate::backend_error::whisper(&e))
                    .wire()
            })?;
            *guard = Some(SpeechRuntime { whisper, vad });
        }
        let runtime = guard.as_mut().unwrap();
        cancel.check()?;
        // VAD sees a peak-normalized copy so quiet speech has the same detection
        // boundary. Whisper always receives the original, unchanged PCM samples.
        let peak = samples.iter().map(|x| x.abs()).fold(0f32, f32::max);
        let detector_samples: Vec<_> = samples
            .iter()
            .map(|x| if peak > 0. { x / peak } else { *x })
            .collect();
        runtime.vad.detect_speech(&detector_samples).map_err(|e| {
            vara_core::UserMessage::new("whisper.transcription")
                .caused_by(crate::backend_error::whisper(&e))
                .wire()
        })?;
        cancel.check()?;
        let mut vad_params = WhisperVadParams::default();
        // Match the existing 100-ms audio minimum instead of adding VAD's
        // default 250-ms hard gate for short utterances.
        vad_params.set_min_speech_duration(100);
        vad_params.set_min_silence_duration(1000);
        vad_params.set_speech_pad(200);
        let regions = runtime
            .vad
            .segments_from_probabilities(vad_params)
            .map_err(|e| {
                vara_core::UserMessage::new("whisper.transcription")
                    .caused_by(crate::backend_error::whisper(&e))
                    .wire()
            })?;
        let mut text = String::new();
        let mut segments = Vec::new();
        for (region_id, region) in regions.enumerate() {
            cancel.check()?;
            // Native VAD coordinates are centiseconds. Bound the source extraction
            // because its final frame can include padding beyond the PCM length.
            let start = (region.start as f64 * 160.).round() as usize;
            let end = ((region.end as f64 * 160.).round() as usize).min(samples.len());
            if start >= end {
                return Err(vara_core::UserMessage::new("transcription.invalid_times").wire());
            }
            let mut state = runtime.whisper.create_state().map_err(|e| {
                vara_core::UserMessage::new("whisper.start")
                    .caused_by(crate::backend_error::whisper(&e))
                    .wire()
            })?;
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_n_threads(
                std::thread::available_parallelism()
                    .map(|n| n.get().min(8))
                    .unwrap_or(4) as i32,
            );
            params.set_language(language);
            params.set_translate(false);
            params.set_no_context(true);
            params.set_no_timestamps(!timed);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);
            params.set_print_special(false);
            // whisper_full is synchronous; cancel's Arc outlives every callback. No ownership crosses FFI.
            unsafe extern "C" fn abort(data: *mut std::ffi::c_void) -> bool {
                unsafe {
                    (*(data as *const std::sync::atomic::AtomicBool))
                        .load(std::sync::atomic::Ordering::SeqCst)
                }
            }
            unsafe {
                params.set_abort_callback(Some(abort));
                params.set_abort_callback_user_data(
                    std::sync::Arc::as_ptr(&cancel.0) as *mut std::ffi::c_void
                );
            }
            let inference = state.full(params, &samples[start..end]);
            // The native abort callback reports a backend error. Accepted cancellation
            // owns the outcome, including when that abort caused full() to fail.
            cancel.check()?;
            inference.map_err(|e| {
                vara_core::UserMessage::new("whisper.transcription")
                    .caused_by(crate::backend_error::whisper(&e))
                    .wire()
            })?;
            for i in 0..state.full_n_segments() {
                if let Some(s) = state.get_segment(i) {
                    if s.no_speech_probability() < 0.85 {
                        let part = s.to_str().map_err(|e| {
                            vara_core::UserMessage::new("local.text_decode")
                                .caused_by(crate::backend_error::whisper(&e))
                                .wire()
                        })?;
                        if !text.is_empty() {
                            text.push(' ');
                        }
                        text.push_str(part.trim());
                        if timed {
                            // Validate local ASR times before offsetting: a bad local
                            // timestamp must not become valid inside a longer file.
                            let local_start = s.start_timestamp() as f64 / 100.;
                            let local_end = s.end_timestamp() as f64 / 100.;
                            validate_region_times(
                                local_start,
                                local_end,
                                (end - start) as f64 / 16000.,
                            )?;
                            segments.push(vara_core::TranscriptSegment {
                                id: format!("r{region_id}-s{i}"),
                                text: part.trim().into(),
                                start: Some(start as f64 / 16000. + local_start),
                                end: Some(start as f64 / 16000. + local_end),
                                speaker: None,
                                words: vec![],
                            });
                        }
                    }
                }
            }
        }
        cancel.check()?;
        if timed {
            Ok(vara_core::Transcript {
                text: text.trim().into(),
                segments,
                timing: vara_core::Timing::Segment,
                timing_source: "asr".into(),
            })
        } else {
            Ok(vara_core::Transcript::plain(text))
        }
    }
    pub fn text(
        &self,
        dir: &Path,
        plan: &vara_core::TextPlan,
        text: &str,
        cancel: &Cancellation,
    ) -> Result<String> {
        let mut guard = self.text.lock();
        cancel.check()?;
        if guard.is_none() {
            let mut file = File::open(dir.join("Qwen3-0.6B-Q8_0.gguf")).map_err(|e| {
                vara_core::UserMessage::new("local.model_missing")
                    .caused_by(vara_core::UserMessage::io_cause(&e))
                    .wire()
            })?;
            let ct = gguf_file::Content::read(&mut file).map_err(|e| {
                vara_core::UserMessage::new("local.model_corrupt")
                    .caused_by(crate::backend_error::candle(&e))
                    .wire()
            })?;
            let weights = ModelWeights::from_gguf(ct, &mut file, &Device::Cpu).map_err(|e| {
                vara_core::UserMessage::new("local.model_load")
                    .caused_by(crate::backend_error::candle(&e))
                    .wire()
            })?;
            let tokenizer =
                tokenizers::Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| {
                    vara_core::UserMessage::new("local.tokenizer_load")
                        .caused_by(crate::backend_error::boxed(e.as_ref(), "backend.tokenizer"))
                        .wire()
                })?;
            *guard = Some((weights, tokenizer));
        }
        let (model, tokenizer) = guard.as_mut().unwrap();
        model.clear_kv_cache();
        // Escape chat control tokens in user-controlled content, preserving the actual transcript separately.
        let escape = |s: &str| s.replace("<|", "< | ").replace("|>", " | >");
        let prompt = format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\nTransskription til bearbejdning:\n{}\n/no_think<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n",
            escape(&plan.system_prompt),
            escape(text)
        );
        let mut tokens = tokenizer
            .encode(prompt, false)
            .map_err(|e| {
                vara_core::UserMessage::new("local.tokenize")
                    .caused_by(crate::backend_error::boxed(e.as_ref(), "backend.tokenizer"))
                    .wire()
            })?
            .get_ids()
            .to_vec();
        if tokens.len() > 3072 {
            return Err(vara_core::UserMessage::new("local.text_too_long").wire());
        }
        let mut sampler = LogitsProcessor::new(42, None, None);
        let mut output = Vec::new();
        let mut offset = 0;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        for _ in 0..1024 {
            cancel.check()?;
            if std::time::Instant::now() > deadline {
                return Err(vara_core::UserMessage::new("local.timeout").wire());
            }
            let input = Tensor::new(tokens.as_slice(), &Device::Cpu)
                .and_then(|t| t.unsqueeze(0))
                .map_err(|e| {
                    vara_core::UserMessage::new("local.model_run")
                        .caused_by(crate::backend_error::candle(&e))
                        .wire()
                })?;
            let logits = model
                .forward(&input, offset)
                .and_then(|t| t.squeeze(0))
                .map_err(|e| {
                    vara_core::UserMessage::new("local.model_run")
                        .caused_by(crate::backend_error::candle(&e))
                        .wire()
                })?;
            offset += tokens.len();
            let token = sampler.sample(&logits).map_err(|e| {
                vara_core::UserMessage::new("local.generation")
                    .caused_by(crate::backend_error::candle(&e))
                    .wire()
            })?;
            if Some(token) == tokenizer.token_to_id("<|im_end|>")
                || Some(token) == tokenizer.token_to_id("<|endoftext|>")
            {
                return tokenizer.decode(&output, true).map_err(|e| {
                    vara_core::UserMessage::new("local.output_decode")
                        .caused_by(crate::backend_error::boxed(e.as_ref(), "backend.tokenizer"))
                        .wire()
                });
            }
            output.push(token);
            tokens = vec![token];
        }
        Err(vara_core::UserMessage::new("ai.text_limit").wire())
    }
}
/// Decode mono PCM and resample to Whisper's 16kHz. Cloud gets this same bounded WAV.
pub(crate) fn read_audio(path: &Path) -> Result<(Vec<f32>, f64)> {
    let mut reader = hound::WavReader::open(path).map_err(|e| {
        vara_core::UserMessage::new("audio.recording_read")
            .caused_by(crate::media::hound_cause(&e))
            .wire()
    })?;
    let spec = reader.spec();
    if spec.sample_format != hound::SampleFormat::Int
        || spec.bits_per_sample != 16
        || spec.channels != 1
        || spec.sample_rate == 0
    {
        return Err(vara_core::UserMessage::new("audio.invalid_format").wire());
    }
    let source: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.map(|v| v as f32 / 32768.0))
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| {
            vara_core::UserMessage::new("audio.corrupt")
                .caused_by(crate::media::hound_cause(&e))
                .wire()
        })?;
    let duration = source.len() as f64 / spec.sample_rate as f64;
    if duration > 600.0 {
        return Err(vara_core::UserMessage::new("audio.too_long").wire());
    }
    if spec.sample_rate == 16000 {
        return Ok((source, duration));
    }
    let n = (duration * 16000.0) as usize;
    let ratio = spec.sample_rate as f64 / 16000.;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let p = i as f64 * ratio;
        let lo = p as usize;
        let hi = (lo + 1).min(source.len().saturating_sub(1));
        let f = (p - lo as f64) as f32;
        out.push(source[lo] * (1. - f) + source[hi] * f);
    }
    Ok((out, duration))
}

fn validate_region_times(start: f64, end: f64, duration: f64) -> Result<()> {
    // Match Transcript::validate's existing 100-ms native timestamp tolerance.
    // Preserve reported times; never clamp out-of-region decoder output.
    if !start.is_finite() || !end.is_finite() || start < 0. || end <= start || end > duration + 0.1
    {
        return Err(vara_core::UserMessage::new("transcription.invalid_times").wire());
    }
    Ok(())
}
#[cfg(test)]
mod region_tests {
    use super::validate_region_times;
    #[test]
    fn local_times_must_fit_region_before_original_timeline_offset() {
        assert!(validate_region_times(0., 8.36, 8.36).is_ok());
        assert!(validate_region_times(0., 8.45, 8.36).is_ok());
        for (start, end) in [
            (0., 8.47),
            (-0.01, 8.),
            (8., 8.),
            (f64::NAN, 8.),
            (0., f64::INFINITY),
        ] {
            assert!(validate_region_times(start, end, 8.36).is_err());
        }
    }
}
