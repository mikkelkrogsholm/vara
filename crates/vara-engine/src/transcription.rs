use crate::{Cancellation, Engines, Result, media, models};
use std::path::Path;
use vara_core::{EngineConfig, Provider, Timing, Transcript};
impl Engines {
    /// File recognition has its own rich result; ordinary dictation keeps its text API.
    pub fn transcribe_file(
        &self,
        engine: &EngineConfig,
        path: &Path,
        language: Option<&str>,
        key: Option<&str>,
        cancel: &Cancellation,
        progress: &(dyn Fn(f64) + Sync),
    ) -> Result<Transcript> {
        engine.validate().map_err(|e| e.to_string())?;
        cancel.check()?;
        if engine.provider == Provider::LocalParakeet {
            return self.parakeet.transcript(
                &models::model_directory(&self.models, "local.parakeet"),
                path,
                cancel,
                progress,
            );
        }
        let mut result = Transcript::plain(String::new());
        media::chunks(path, cancel, |chunk, offset, duration| {
            let mut part = if engine.provider == Provider::LocalWhisper {
                self.local.transcript(
                    &self.models.join("ggml-base.bin"),
                    chunk,
                    language,
                    cancel,
                )?
            } else {
                tokio::runtime::Handle::current().block_on(crate::remote::file_transcript(
                    &self.client,
                    engine,
                    chunk,
                    language,
                    key,
                    cancel,
                ))?
            };
            cancel.check()?;
            if !result.text.is_empty() {
                result.text.push(' ');
            }
            result.text += part.text.trim();
            for segment in &mut part.segments {
                segment.id = format!("c{}-{}", offset as u64, segment.id);
                if let Some(speaker) = &mut segment.speaker {
                    *speaker = format!("{}:{}", offset as u64, speaker);
                }

                if let Some(t) = &mut segment.start {
                    *t += offset;
                }
                if let Some(t) = &mut segment.end {
                    *t += offset;
                }
                for word in &mut segment.words {
                    if let Some(speaker) = &mut word.speaker {
                        *speaker = format!("{}:{}", offset as u64, speaker);
                    }
                    word.id = format!("c{}-{}", offset as u64, word.id);
                    word.start += offset;
                    word.end += offset;
                }
            }
            if offset == 0. {
                result.segments.clear();
                result.timing = part.timing;
                result.timing_source = part.timing_source;
            }
            result.segments.extend(part.segments);
            progress(offset + duration);
            Ok(())
        })?;
        if result.text.trim().is_empty() {
            return Err("Ingen tale genkendt i filen.".into());
        }
        if result.timing == Timing::None {
            result.timing_source = "none".into();
        }
        Ok(result)
    }
}
