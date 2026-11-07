//! Opt-in GUI acceptance timing; recognition, target capture and delivery remain real.
use super::*;
use std::{sync::atomic::Ordering, time::Duration};

fn timings(delay: &str, recording: &str) -> Result<(Duration, Duration)> {
    let parse = |value: &str, min| {
        value
            .parse::<u64>()
            .ok()
            .filter(|n| (min..=30_000).contains(n))
            .map(Duration::from_millis)
            .ok_or_else(|| "GUI fixture timing is out of bounds".to_owned())
    };
    Ok((parse(delay, 1_000)?, parse(recording, 500)?))
}

fn local_settings(settings: &Settings) -> bool {
    settings.strict_local
        && settings.speech_engine == "local.parakeet"
        && settings.engine("local.parakeet").is_ok_and(|e| {
            e.provider == Provider::LocalParakeet && e.locality == Locality::OnDevice
        })
        && !settings.general_ai_enabled
        && settings.app_rules.values().all(|r| !r.enabled)
}

fn textedit_target(origin: &Destination) -> bool {
    origin.app_id == "com.apple.TextEdit" && origin.target_token.is_some()
}

impl Service {
    fn gui_fixture_timing(&self, settings: &Settings) -> Result<Option<(Duration, Duration)>> {
        let Some(delay) = std::env::var_os("VARA_TEST_GUI_START_DELAY_MS") else {
            return Ok(None);
        };
        let root = std::env::var_os("VARA_DATA_DIR")
            .map(PathBuf::from)
            .and_then(|p| p.canonicalize().ok());
        let actual = self.audio.parent().and_then(|p| p.canonicalize().ok());
        let temp = PathBuf::from("/tmp")
            .canonicalize()
            .map_err(|_| "GUI fixture requires temporary data")?;
        if root
            .as_ref()
            .is_none_or(|p| !p.starts_with(&temp) || p == &temp)
            || root != actual
            || self.fixture.as_ref().is_none_or(|p| !p.is_file())
            || self.scripted.is_some()
            || !local_settings(settings)
        {
            return Err(
                "GUI fixture requires isolated temporary data, WAV and local-only settings".into(),
            );
        }
        let recording = std::env::var("VARA_TEST_GUI_RECORD_MS")
            .map_err(|_| "GUI fixture recording interval missing")?;
        timings(&delay.to_string_lossy(), &recording).map(Some)
    }

    pub(super) fn guard_gui_fixture(
        &self,
        origin: &Destination,
        settings: &Settings,
    ) -> Result<()> {
        if self.gui_fixture_timing(settings)?.is_some() && !textedit_target(origin) {
            return Err("GUI fixture requires a focused TextEdit input".into());
        }
        Ok(())
    }

    pub(super) fn schedule_gui_fixture(self: &Arc<Self>) -> Result<bool> {
        let Some((delay, recording)) = self.gui_fixture_timing(&self.preferences())? else {
            return Ok(false);
        };
        if self.inner.lock().active.is_some() {
            return Err(Error::Busy.wire());
        }
        let epoch = self.gui_fixture_epoch.fetch_add(1, Ordering::SeqCst) + 1;
        let service = self.clone();
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            if service.gui_fixture_epoch.load(Ordering::SeqCst) != epoch {
                return;
            }
            if let Err(error) = service.start(ProcessingChoice::Direct) {
                service.warn(error);
                return;
            }
            let generation = service
                .inner
                .lock()
                .active
                .as_ref()
                .map(|a| a.session.generation.clone());
            std::thread::sleep(recording);
            if service.gui_fixture_epoch.load(Ordering::SeqCst) != epoch {
                return;
            }
            let same = service
                .inner
                .lock()
                .active
                .as_ref()
                .is_some_and(|a| Some(&a.session.generation) == generation.as_ref());
            if same && let Err(error) = service.stop() {
                service.warn(error);
            }
        });
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_timers_are_bounded() {
        assert!(timings("6000", "1000").is_ok());
        for (delay, recording) in [
            ("0", "1000"),
            ("6000", "0"),
            ("30001", "1000"),
            ("6000", "30001"),
            ("bad", "1000"),
        ] {
            assert!(timings(delay, recording).is_err());
        }
    }
    #[test]
    fn fixture_requires_real_textedit_target() {
        let mut origin = Destination {
            app_id: "com.apple.TextEdit".into(),
            app_name: "TextEdit".into(),
            target_token: Some("actual-native-token".into()),
        };
        assert!(textedit_target(&origin));
        origin.app_id = "dk.vara.desktop".into();
        assert!(!textedit_target(&origin));
        origin.app_id = "com.apple.TextEdit".into();
        origin.target_token = None;
        assert!(!textedit_target(&origin));
    }
    #[test]
    fn fixture_requires_local_direct_settings() {
        let mut settings = Settings::default();
        assert!(!local_settings(&settings));
        settings.strict_local = true;
        assert!(local_settings(&settings));
        settings.general_ai_enabled = true;
        assert!(!local_settings(&settings));
        settings.general_ai_enabled = false;
        settings.speech_engine = "cloud.gemini".into();
        assert!(!local_settings(&settings));
        settings.speech_engine = "local.parakeet".into();
        settings
            .engines
            .iter_mut()
            .find(|e| e.id == "local.parakeet")
            .unwrap()
            .provider = Provider::Gemini;
        assert!(!local_settings(&settings));
    }
}
