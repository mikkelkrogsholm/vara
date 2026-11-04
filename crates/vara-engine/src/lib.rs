//! Runtime adapters. Requests take immutable core plans; cancellation never changes providers.
pub mod cli;
mod local;
pub mod media;
pub mod models;
mod parakeet;
mod remote;
mod transcription;
pub mod who;
pub use models::*;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use vara_core::{EngineConfig, Provider, TextPlan};

pub type Result<T> = std::result::Result<T, String>;
/// Keep actionable transport classification and HTTP status without URLs or response bodies.
pub(crate) fn network_cause(error: &reqwest::Error) -> vara_core::UserMessage {
    if let Some(status) = error.status() {
        return vara_core::UserMessage::new("provider.http")
            .number("status", status.as_u16().into());
    }
    let code = if error.is_timeout() {
        "provider.timeout"
    } else if error.is_connect() {
        "provider.connect"
    } else {
        "provider.transport"
    };
    vara_core::UserMessage::new(code)
}

fn process_cause(status: &std::process::ExitStatus) -> vara_core::UserMessage {
    if let Some(code) = status.code() {
        return vara_core::UserMessage::new("process.exit_code").number("code", code.into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return vara_core::UserMessage::new("process.signal").number("signal", signal.into());
        }
    }
    vara_core::UserMessage::new("process.unknown")
}
#[derive(Clone, Default)]
pub struct Cancellation(
    pub Arc<AtomicBool>,
    Arc<parking_lot::Mutex<bool>>,
    Arc<parking_lot::Mutex<Vec<std::sync::Weak<parking_lot::Mutex<std::process::Child>>>>>,
);
impl Cancellation {
    pub fn cancel(&self) -> bool {
        let finished = self.1.lock();
        if *finished {
            return false;
        }
        self.0.store(true, Ordering::SeqCst);
        for child in self.2.lock().iter().filter_map(std::sync::Weak::upgrade) {
            let mut child = child.lock();
            let _ = child.kill();
            let _ = child.wait();
        }
        true
    }
    /// Register local subprocesses under the cancellation gate, so quit cannot orphan them.
    pub(crate) fn spawn_child(
        &self,
        spawn: impl FnOnce() -> std::io::Result<std::process::Child>,
    ) -> Result<Arc<parking_lot::Mutex<std::process::Child>>> {
        let _gate = self.1.lock();
        self.check()?;
        let child = Arc::new(parking_lot::Mutex::new(spawn().map_err(|e| {
            vara_core::UserMessage::new("media.decoder_start")
                .caused_by(vara_core::UserMessage::io_cause(&e))
                .wire()
        })?));
        self.2.lock().push(Arc::downgrade(&child));
        Ok(child)
    }
    /// Serialize the installer publication boundary with accepted cancellation.
    pub fn publish<T>(&self, work: impl FnOnce() -> Result<T>) -> Result<T> {
        let mut finished = self.1.lock();
        self.check()?;
        let value = work()?;
        *finished = true;
        Ok(value)
    }
    pub fn check(&self) -> Result<()> {
        if self.0.load(Ordering::SeqCst) {
            Err(vara_core::UserMessage::new("runtime.cancelled").wire())
        } else {
            Ok(())
        }
    }
}
pub struct Engines {
    pub models: PathBuf,
    client: reqwest::Client,
    local: local::Local,
    parakeet: parakeet::Parakeet,
}
impl Engines {
    pub fn new(models: PathBuf) -> Result<Self> {
        whisper_rs::install_logging_hooks();
        Ok(Self {
            models,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .redirect(reqwest::redirect::Policy::none())
                .no_proxy()
                .build()
                .map_err(|e| {
                    vara_core::UserMessage::new("provider.client_start")
                        .caused_by(network_cause(&e))
                        .wire()
                })?,
            local: local::Local::default(),
            parakeet: parakeet::Parakeet::default(),
        })
    }
    pub async fn speech(
        &self,
        engine: &EngineConfig,
        path: &Path,
        language: Option<&str>,
        key: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<String> {
        cancel.check()?;
        engine.validate().map_err(|e| e.wire())?;
        let text = match engine.provider {
            Provider::LocalParakeet => self.parakeet.speech(
                &models::model_directory(&self.models, "local.parakeet"),
                path,
                cancel,
            )?,
            Provider::LocalWhisper => {
                self.local
                    .speech(&self.models.join("ggml-base.bin"), path, language, cancel)?
            }
            Provider::OpenAi | Provider::Groq | Provider::ElevenLabs => {
                remote::speech(&self.client, engine, path, language, key, cancel).await?
            }
            _ => return Err(vara_core::UserMessage::new("engine.speech_unsupported").wire()),
        };
        cancel.check()?;
        let text = text.trim().to_string();
        if text.is_empty() {
            Err(vara_core::UserMessage::new("engine.no_speech").wire())
        } else {
            Ok(text)
        }
    }
    pub async fn text(
        &self,
        plan: &TextPlan,
        text: &str,
        key: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<String> {
        cancel.check()?;
        plan.engine.validate().map_err(|e| e.wire())?;
        let result = match plan.engine.provider {
            Provider::LocalText => self.local.text(&self.models, plan, text, cancel)?,
            Provider::ClaudeCli | Provider::CodexCli | Provider::GeminiCli => {
                cli::text(plan, text, cancel).await?
            }
            _ => remote::text(&self.client, plan, text, key, cancel).await?,
        };
        cancel.check()?;
        if result.trim().is_empty() {
            Err(vara_core::UserMessage::new("ai.empty_original_preserved").wire())
        } else {
            Ok(result.trim().into())
        }
    }
    /// Checks the saved credential and visible model catalogue; sends no audio/text.
    pub async fn openai_models(&self, key: &str, cancel: &Cancellation) -> Result<Vec<String>> {
        remote::openai_models(&self.client, key, cancel).await
    }
    pub async fn groq_models(&self, key: &str, cancel: &Cancellation) -> Result<Vec<String>> {
        remote::groq_models(&self.client, key, cancel).await
    }
    pub fn ready(&self, id: &str) -> bool {
        models::artifacts(id).is_some_and(|a| {
            a.iter().all(|a| {
                std::fs::metadata(models::model_directory(&self.models, id).join(a.name))
                    .is_ok_and(|m| m.len() == a.bytes)
            })
        })
    }
}

/// Read duration without decoding or exposing audio bytes to the UI.
pub fn audio_duration(path: &Path) -> Result<f64> {
    let reader = hound::WavReader::open(path)
        .map_err(|_| vara_core::UserMessage::new("audio.recording_read").wire())?;
    let rate = reader.spec().sample_rate;
    if rate == 0 {
        return Err(vara_core::UserMessage::new("audio.invalid_format").wire());
    }
    Ok(reader.duration() as f64 / rate as f64)
}

mod who_consensus;
mod who_known;
