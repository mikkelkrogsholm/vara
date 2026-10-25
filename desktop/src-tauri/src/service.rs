use parking_lot::Mutex;
use serde::Serialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tauri::{AppHandle, Emitter};
use vara_core::*;
use vara_engine::{Cancellation, Engines, Progress};
use vara_platform as platform;
type Result<T> = std::result::Result<T, String>;
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
#[derive(Clone)]
struct Active {
    session: Session,
    cancel: Cancellation,
    path: PathBuf,
    duration: f64,
}
struct Inner {
    store: Store,
    settings: Settings,
    next_choice: ProcessingChoice,
    active: Option<Active>,
    unsaved: Option<Dictation>,
    message: Option<String>,
    downloads: BTreeMap<String, (Cancellation, Progress)>,
}
pub struct Service {
    inner: Mutex<Inner>,
    pub settings_gate: Mutex<()>,
    pub engines: Arc<Engines>,
    recorder: platform::Recorder,
    audio: PathBuf,
    app: AppHandle,
    #[cfg(feature = "test-harness")]
    fixture: Option<PathBuf>,
}
#[derive(Clone, Serialize)]
pub struct Snapshot {
    settings: Settings,
    next_choice: ProcessingChoice,
    history: Vec<Dictation>,
    recoveries: Vec<Recovery>,
    permissions: platform::Permissions,
    session: Option<Session>,
    unsaved: Option<Dictation>,
    message: Option<String>,
    models: BTreeMap<String, bool>,
    downloads: Vec<Progress>,
}
impl Service {
    pub fn new(app: AppHandle, root: PathBuf) -> Result<Arc<Self>> {
        std::fs::create_dir_all(root.join("audio"))
            .map_err(|_| "Vara kunne ikke oprette sin datamappe.")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
                .map_err(|_| "Datamappen kunne ikke beskyttes.")?;
            std::fs::set_permissions(root.join("audio"), std::fs::Permissions::from_mode(0o700))
                .map_err(|_| "Lydmappen kunne ikke beskyttes.")?;
        }
        let mut store = Store::open(&root.join("vara.sqlite3")).map_err(|e| e.to_string())?;
        let settings = store.settings().map_err(|e| e.to_string())?;
        settings.validate().map_err(|e| e.to_string())?;
        let mut recovered = false;
        // Interrupted post-ASR work is recovered as original text, never reprocessed or dispatched automatically.
        for stage in store.staged().map_err(|e| e.to_string())? {
            store
                .commit(&Dictation {
                    id: uuid::Uuid::new_v4().to_string(),
                    generation: stage.generation,
                    origin: stage.origin,
                    created_at: stage.created_at,
                    original: stage.original,
                    processed: None,
                    fallback: true,
                    delivery: Delivery::ManualReady,
                    plan: stage.plan,
                    duration_secs: 0.,
                })
                .map_err(|e| e.to_string())?;
            recovered = true;
        }
        let service=Arc::new(Self{#[cfg(feature="test-harness")] fixture:std::env::var_os("VARA_TEST_WAV").map(PathBuf::from),settings_gate:Mutex::new(()),engines:Arc::new(Engines::new(root.join("models"))?),recorder:platform::Recorder::new(),audio:root.join("audio"),inner:Mutex::new(Inner{store,settings,next_choice:ProcessingChoice::Default,active:None,unsaved:None,message:recovered.then(||"Tekst fra en afbrudt diktering er gendannet i historikken. Indsæt den manuelt.".into()),downloads:BTreeMap::new()}),app});
        service.maintenance()?;
        Ok(service)
    }
    pub(crate) fn preferences(&self) -> Settings {
        self.inner.lock().settings.clone()
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let i = self.inner.lock();
        Ok(Snapshot {
            settings: i.settings.clone(),
            next_choice: i.next_choice,
            history: i.store.history(None, None).map_err(|e| e.to_string())?,
            recoveries: i
                .store
                .recoveries()
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter(|r| r.failed && r.eligible(now()))
                .collect(),
            permissions: platform::permissions(),
            session: i.active.as_ref().map(|a| a.session.clone()),
            unsaved: i.unsaved.clone(),
            message: i.message.clone(),
            models: ["local.parakeet", "local.whisper", "local.text"]
                .into_iter()
                .map(|id| (id.into(), self.engines.ready(id)))
                .collect(),
            downloads: i.downloads.values().map(|(_, p)| p.clone()).collect(),
        })
    }
    fn show_hud(&self, title: &str, recording: bool) {
        let i = self.inner.lock();
        if let Some(active) = &i.active {
            let location = |place: Locality| match place {
                Locality::OnDevice => "lokal",
                Locality::Lan => "eget netværk",
                Locality::Cloud => "cloud",
                Locality::Unknown => "ekstern/ukendt",
            };
            let detail = if active.session.stage == Stage::Processing {
                active
                    .session
                    .plan
                    .text
                    .as_ref()
                    .map(|p| {
                        format!(
                            "AI: {} · {}",
                            location(p.engine.locality),
                            if p.app_specific {
                                "app-prompt"
                            } else {
                                "generel"
                            }
                        )
                    })
                    .unwrap_or_default()
            } else {
                format!(
                    "Lyd: {}",
                    if active.session.plan.speech.locality == Locality::OnDevice {
                        "på enheden"
                    } else {
                        location(active.session.plan.speech.locality)
                    }
                )
            };
            platform::hud(
                platform::Hud {
                    origin: &active.session.origin,
                    title,
                    detail: &detail,
                    recording,
                },
                crate::hud_callbacks(),
            );
        }
    }
    pub fn emit(&self) {
        if let Ok(snapshot) = self.snapshot() {
            let _ = self.app.emit("vara:state", snapshot);
        }
    }
    pub fn warn(&self, message: String) {
        self.inner.lock().message = Some(message);
        self.emit();
    }
    pub fn level(&self) -> f32 {
        self.recorder.level()
    }
    pub fn is_recording(&self) -> bool {
        self.inner
            .lock()
            .active
            .as_ref()
            .is_some_and(|a| a.session.stage == Stage::Recording)
    }
    pub fn activation(&self) -> Activation {
        self.inner.lock().settings.activation
    }
    pub fn shortcut(&self) -> String {
        self.inner.lock().settings.shortcut.clone()
    }
    pub fn start(self: &Arc<Self>, choice: ProcessingChoice) -> Result<()> {
        let mut i = self.inner.lock();
        if i.active.is_some() {
            return Err(Error::Busy.to_string());
        }
        if i.unsaved.is_some() {
            return Err("Gem eller kopiér først teksten, der endnu ikke er gemt.".into());
        }
        let permissions = platform::permissions();
        if permissions.microphone == "not_determined" || permissions.microphone == "denied" {
            return Err("Giv Vara adgang til mikrofonen i Indstillinger, før du dikterer.".into());
        }
        let choice = if choice == ProcessingChoice::Default {
            i.next_choice
        } else {
            choice
        };
        let origin = platform::capture_target()?;
        let plan = Plan::resolve(&i.settings, &origin, choice).map_err(|e| e.to_string())?;
        if matches!(
            plan.speech.provider,
            Provider::LocalWhisper | Provider::LocalParakeet
        ) && !self.engines.ready(&plan.speech.id)
        {
            platform::release_target(&origin);
            return Err("Hent først den lokale talemodel i Indstillinger → Forbindelser.".into());
        }
        if needs_key(plan.speech.provider) && platform::secret(&key_id(&plan.speech))?.is_none() {
            platform::release_target(&origin);
            return Err("Tilføj først en API-nøgle til den valgte taleudbyder.".into());
        }
        let session = Session::new(origin, plan, now());
        let path = self.audio.join(format!("{}.wav", session.recovery_id));
        let recovery = Recovery {
            id: session.recovery_id.clone(),
            generation: session.generation.clone(),
            origin: session.origin.clone(),
            plan: session.plan.clone(),
            created_at: session.created_at,
            expires_at: session.created_at + AUDIO_TTL_SECS,
            file_name: format!("{}.wav", session.recovery_id),
            failed: false,
            recognized: false,
        };
        i.store.put_recovery(&recovery).map_err(|e| e.to_string())?;
        #[cfg(feature = "test-harness")]
        let capture_started = if let Some(fixture) = &self.fixture {
            std::fs::copy(fixture, &path)
                .map(|_| ())
                .map_err(|_| "Testfixture kunne ikke åbnes.".to_string())
        } else {
            self.recorder
                .start(path.clone(), i.settings.input_device.clone())
        };
        #[cfg(not(feature = "test-harness"))]
        let capture_started = self
            .recorder
            .start(path.clone(), i.settings.input_device.clone());
        if let Err(error) = capture_started {
            let _ = i.store.delete_recovery(&session.recovery_id);
            platform::release_target(&session.origin);
            return Err(error);
        }
        i.message = None;
        i.next_choice = ProcessingChoice::Default;
        i.active = Some(Active {
            session,
            cancel: Cancellation::default(),
            path,
            duration: 0.,
        });
        drop(i);
        self.show_hud("Lytter", true);
        self.emit();
        Ok(())
    }
    pub fn stop(self: &Arc<Self>) -> Result<()> {
        let mut i = self.inner.lock();
        let active = i.active.as_mut().ok_or("Der er ingen aktiv diktering.")?;
        if active.session.stage != Stage::Recording {
            return Ok(());
        }
        #[cfg(feature = "test-harness")]
        let captured = if self.fixture.is_some() {
            vara_engine::audio_duration(&active.path)
        } else {
            self.recorder.stop(true)
        };
        #[cfg(not(feature = "test-harness"))]
        let captured = self.recorder.stop(true);
        match captured {
            Ok(duration) => active.duration = duration,
            Err(error) => {
                let active = i.active.take().unwrap();
                let _ = i.store.delete_recovery(&active.session.recovery_id);
                let _ = std::fs::remove_file(active.path);
                platform::release_target(&active.session.origin);
                drop(i);
                platform::hide_hud();
                return Err(error);
            }
        }
        active
            .session
            .advance(Stage::Recording, Stage::Recognizing)
            .map_err(|e| e.to_string())?;
        let active = active.clone();
        drop(i);
        self.show_hud("Finder dine ord", false);
        self.emit();
        self.launch(active);
        Ok(())
    }
    fn launch(self: &Arc<Self>, active: Active) {
        let service = self.clone();
        tauri::async_runtime::spawn_blocking(move || {
            tauri::async_runtime::block_on(service.pipeline(active));
        });
    }
    fn current(i: &Inner, a: &Active) -> bool {
        i.active
            .as_ref()
            .is_some_and(|v| v.session.generation == a.session.generation)
            && a.cancel.check().is_ok()
    }
    async fn pipeline(self: Arc<Self>, a: Active) {
        let speech_key = engine_secret(&a.session.plan.speech);
        let speech = match speech_key {
            Ok(key) => {
                self.engines
                    .speech(
                        &a.session.plan.speech,
                        &a.path,
                        a.session.plan.language.as_deref(),
                        key.as_deref(),
                        &a.cancel,
                    )
                    .await
            }
            Err(e) => Err(e),
        };
        let original = match speech {
            Ok(text) => text,
            Err(error) => {
                let mut i = self.inner.lock();
                if !Self::current(&i, &a) {
                    return;
                }
                if let Ok(Some(mut recovery)) = i.store.recovery(&a.session.recovery_id) {
                    recovery.failed = true;
                    let _ = i.store.put_recovery(&recovery);
                }
                i.active = None;
                i.message = Some(error);
                drop(i);
                platform::release_target(&a.session.origin);
                platform::hide_hud();
                self.emit();
                return;
            }
        };
        // Audio is no longer needed once ASR succeeds, regardless of AI, disk or delivery outcomes.
        {
            let mut i = self.inner.lock();
            if !Self::current(&i, &a) {
                return;
            }
            let stage = StagedText {
                generation: a.session.generation.clone(),
                origin: a.session.origin.clone(),
                plan: a.session.plan.clone(),
                original: original.clone(),
                created_at: a.session.created_at,
            };
            let staged = i.store.stage(&stage);
            if let Ok(Some(mut recovery)) = i.store.recovery(&a.session.recovery_id) {
                recovery.recognized = true;
                recovery.failed = false;
                let _ = i.store.put_recovery(&recovery);
            }
            if std::fs::remove_file(&a.path).is_err() && a.path.exists() {
                i.message = Some("Lyden kunne ikke slettes. Vara prøver oprydning igen.".into());
            } else {
                let _ = i.store.delete_recovery(&a.session.recovery_id);
            }
            if staged.is_err() {
                i.unsaved = Some(result(&a, original, None, true));
                i.active = None;
                i.message =
                    Some("Teksten kunne ikke gemmes. Kopiér den, eller prøv at gemme igen.".into());
                drop(i);
                platform::release_target(&a.session.origin);
                platform::hide_hud();
                self.emit();
                return;
            }
            if a.session.plan.text.is_some() {
                if let Some(active) = i.active.as_mut() {
                    active.session.stage = Stage::Processing;
                }
            }
        }
        let mut fallback = false;
        let mut ai_error = None;
        let processed = if let Some(plan) = &a.session.plan.text {
            self.show_hud("Tilpasser teksten", false);
            self.emit();
            let key = engine_secret(&plan.engine);
            let output = match key {
                Ok(key) => {
                    self.engines
                        .text(plan, &original, key.as_deref(), &a.cancel)
                        .await
                }
                Err(error) => Err(error),
            };
            match output {
                Ok(text) => Some(text),
                Err(error) => {
                    fallback = true;
                    ai_error = Some(error);
                    None
                }
            }
        } else {
            None
        };
        let mut i = self.inner.lock();
        if !Self::current(&i, &a) {
            return;
        }
        let mut r = result(&a, original, processed, fallback);
        if let Some(active) = i.active.as_mut() {
            active.session.stage = Stage::Committing;
        }
        match i.store.commit(&r) {
            Ok(saved) => r = saved,
            Err(_) => {
                i.unsaved = Some(r);
                i.message =
                    Some("Teksten kunne ikke gemmes. Kopiér den, eller prøv at gemme igen.".into());
                i.active = None;
                drop(i);
                platform::release_target(&a.session.origin);
                platform::hide_hud();
                self.emit();
                return;
            }
        }
        // Gate serializes cancellation and commit. Unknown is durable BEFORE the OS call.
        if i.store.update_delivery(&r.id, Delivery::Unknown).is_ok() {
            let delivery = platform::insert(&r.origin, r.finished_text());
            if let Err(error) = i.store.update_delivery(&r.id, delivery) {
                i.message = Some(error.to_string());
            }
        } else {
            i.message = Some("Teksten er gemt og klar til manuel indsættelse.".into());
        }
        i.active = None;
        if let Some(error) = ai_error {
            i.message = Some(format!(
                "AI-bearbejdning mislykkedes. Originalen er bevaret. {error}"
            ));
        }
        i.settings.onboarding_complete = true;
        let _ = i.store.save_settings(&i.settings);
        drop(i);
        platform::release_target(&a.session.origin);
        platform::hide_hud();
        self.emit();
    }
    pub fn cancel(&self) -> Result<CancelOutcome> {
        let mut i = self.inner.lock();
        let Some(active) = i.active.as_mut() else {
            return Ok(CancelOutcome::AlreadyCancelled);
        };
        let outcome = active.session.cancel();
        if outcome != CancelOutcome::Cancelled {
            return Ok(outcome);
        }
        let active = i.active.take().unwrap();
        let durable_cancel = i
            .store
            .cancel_generation(&active.session.generation)
            .map_err(|e| e.to_string());
        active.cancel.cancel();
        let _ = self.recorder.stop(false);
        let _ = std::fs::remove_file(&active.path);
        let _ = i.store.delete_recovery(&active.session.recovery_id);
        let _ = i.store.remove_staged(&active.session.generation);
        drop(i);
        platform::release_target(&active.session.origin);
        platform::hide_hud();
        self.emit();
        durable_cancel?;
        Ok(outcome)
    }
    pub fn retry(self: &Arc<Self>, id: &str) -> Result<()> {
        self.maintenance()?;
        let mut i = self.inner.lock();
        if i.active.is_some() || i.unsaved.is_some() {
            return Err(Error::Busy.to_string());
        }
        let r = i
            .store
            .recovery(id)
            .map_err(|e| e.to_string())?
            .ok_or("Optagelsen findes ikke længere.")?;
        if !r.eligible(now()) {
            return Err(Error::Expired.to_string());
        }
        r.plan.check_locations().map_err(|e| e.to_string())?;
        // A retry cannot safely infer that the old field has stayed focused across failure/restart.
        let mut origin = r.origin.clone();
        origin.target_token = None;
        let active = Active {
            session: Session {
                generation: uuid::Uuid::new_v4().to_string(),
                origin,
                plan: r.plan,
                stage: Stage::Recognizing,
                created_at: r.created_at,
                recovery_id: r.id.clone(),
            },
            cancel: Cancellation::default(),
            path: self.audio.join(&r.file_name),
            duration: 0.,
        };
        if !active.path.is_file() {
            i.store.delete_recovery(id).map_err(|e| e.to_string())?;
            return Err("Lydfilen findes ikke længere.".into());
        }
        i.active = Some(active.clone());
        i.message = None;
        drop(i);
        self.emit();
        self.launch(active);
        Ok(())
    }
    pub fn delete_recovery(&self, id: &str) -> Result<()> {
        let i = self.inner.lock();
        if i.active
            .as_ref()
            .is_some_and(|a| a.session.recovery_id == id)
        {
            return Err("Annuller først genprøvningen.".into());
        }
        if let Some(r) = i.store.recovery(id).map_err(|e| e.to_string())? {
            std::fs::remove_file(self.audio.join(r.file_name))
                .or_else(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Ok(())
                    } else {
                        Err(e)
                    }
                })
                .map_err(|_| "Optagelsen kunne ikke slettes.")?;
            i.store.delete_recovery(id).map_err(|e| e.to_string())?;
        }
        drop(i);
        self.emit();
        Ok(())
    }
    pub fn maintenance(&self) -> Result<()> {
        let mut i = self.inner.lock();
        let current_time = now();
        let days = i.settings.retention_days;
        i.store
            .retain(current_time, days)
            .map_err(|e| e.to_string())?;
        for mut r in i.store.recoveries().map_err(|e| e.to_string())? {
            if current_time >= r.expires_at || r.recognized {
                if i.active
                    .as_ref()
                    .is_some_and(|a| a.session.recovery_id == r.id && !r.recognized)
                {
                    let active = i.active.take().unwrap();
                    active.cancel.cancel();
                    let _ = self.recorder.stop(false);
                    platform::release_target(&active.session.origin);
                    platform::hide_hud();
                }
                let path = self.audio.join(&r.file_name);
                if path.exists() {
                    std::fs::remove_file(path)
                        .map_err(|_| "En udløbet optagelse kunne ikke slettes.")?;
                }
                i.store.delete_recovery(&r.id).map_err(|e| e.to_string())?;
            } else if !r.failed
                && i.active
                    .as_ref()
                    .is_none_or(|a| a.session.recovery_id != r.id)
            {
                if self.audio.join(&r.file_name).exists() {
                    r.failed = true;
                    i.store.put_recovery(&r).map_err(|e| e.to_string())?;
                } else {
                    i.store.delete_recovery(&r.id).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }
    pub fn save_settings(&self, settings: Settings) -> Result<()> {
        settings.validate().map_err(|e| e.to_string())?;
        let mut i = self.inner.lock();
        i.store
            .save_settings(&settings)
            .map_err(|e| e.to_string())?;
        i.settings = settings;
        drop(i);
        self.maintenance()?;
        self.emit();
        Ok(())
    }
    pub fn install(self: &Arc<Self>, id: String) -> Result<()> {
        if vara_engine::artifacts(&id).is_none() {
            return Err("Ukendt model.".into());
        }
        let cancel = Cancellation::default();
        let mut i = self.inner.lock();
        if i.downloads.contains_key(&id) {
            return Err("Modellen hentes allerede.".into());
        }
        i.downloads.insert(
            id.clone(),
            (
                cancel.clone(),
                Progress {
                    id: id.clone(),
                    received: 0,
                    total: 0,
                    phase: "downloading".into(),
                },
            ),
        );
        drop(i);
        self.emit();
        let service = self.clone();
        tauri::async_runtime::spawn(async move {
            let progress_service = service.clone();
            let progress_id = id.clone();
            let output = vara_engine::install(&id, &service.engines.models, &cancel, move |p| {
                if let Some((_, v)) = progress_service
                    .inner
                    .lock()
                    .downloads
                    .get_mut(&progress_id)
                {
                    *v = p;
                }
                progress_service.emit();
            })
            .await;
            let mut i = service.inner.lock();
            i.downloads.remove(&id);
            if let Err(error) = output {
                i.message = Some(error);
            }
            drop(i);
            service.emit();
        });
        Ok(())
    }
    pub fn command(self: &Arc<Self>, action: &str, p: Value) -> Result<Value> {
        let id = || p["id"].as_str().ok_or_else(|| "Id mangler.".to_string());
        match action {
            "snapshot" => {
                self.maintenance()?;
                return serde_json::to_value(self.snapshot()?)
                    .map_err(|_| "Tilstanden kunne ikke læses.".into());
            }
            "applications" => return platform::applications(),
            "application_icons" => {
                let ids: Vec<String> =
                    serde_json::from_value(p["ids"].clone()).map_err(|_| "Ugyldige app-id’er.")?;
                return platform::application_icons(&ids);
            }
            "set_next_choice" => {
                self.inner.lock().next_choice =
                    serde_json::from_value(p["choice"].clone()).map_err(|_| "Ugyldigt valg.")?;
            }
            "check_cli" => {
                let provider: Provider =
                    serde_json::from_value(p["provider"].clone()).map_err(|_| "Ukendt CLI.")?;
                return serde_json::to_value(tauri::async_runtime::block_on(
                    vara_engine::cli::status(provider),
                ))
                .map_err(|_| "CLI-status kunne ikke læses.".into());
            }
            "devices" => {
                return serde_json::to_value(platform::input_devices()?)
                    .map_err(|_| "Mikrofonerne kunne ikke læses.".into());
            }
            "start" => self.start(
                serde_json::from_value(p.get("choice").cloned().unwrap_or(json!("default")))
                    .map_err(|_| "Ugyldigt valg.")?,
            )?,
            "stop" => self.stop()?,
            "cancel" => return Ok(json!(self.cancel()?)),
            "retry" => self.retry(id()?)?,
            "delete_recovery" => self.delete_recovery(id()?)?,
            "delete_result" => {
                self.inner
                    .lock()
                    .store
                    .delete(id()?)
                    .map_err(|e| e.to_string())?;
            }
            "copy" => {
                let i = self.inner.lock();
                let r = if p["unsaved"] == true {
                    i.unsaved.clone()
                } else {
                    i.store.result(id()?).map_err(|e| e.to_string())?
                }
                .ok_or("Teksten findes ikke længere.")?;
                platform::copy(if p["original"] == true {
                    &r.original
                } else {
                    r.finished_text()
                })?;
            }
            "save_unsaved" => {
                let mut i = self.inner.lock();
                if let Some(mut r) = i.unsaved.clone() {
                    r.delivery = Delivery::ManualReady;
                    i.store.commit(&r).map_err(|e| e.to_string())?;
                    i.unsaved = None;
                    i.message = None;
                }
            }
            "discard_unsaved" => {
                let mut i = self.inner.lock();
                if let Some(r) = i.unsaved.take() {
                    i.store
                        .remove_staged(&r.generation)
                        .map_err(|e| e.to_string())?;
                }
            }
            "permission" => platform::request_permission(p["kind"].as_str().unwrap_or(""))?,
            "install" => self.install(id()?.into())?,
            "cancel_install" => {
                if let Some((cancel, _)) = self.inner.lock().downloads.get(id()?) {
                    if !cancel.cancel() {
                        return Err("Modellen er allerede færdig med at installere.".into());
                    }
                }
            }
            "check_groq" | "check_openai" => {
                let openai = action == "check_openai";
                let provider = if openai {
                    Provider::OpenAi
                } else {
                    Provider::Groq
                };
                let prefix = if openai { "openai" } else { "groq" };
                let key = platform::secret(if openai { "open_ai" } else { "groq" })?
                    .ok_or("Gem først din API-nøgle.")?;
                let available = tauri::async_runtime::block_on(async {
                    if openai {
                        self.engines
                            .openai_models(&key, &Cancellation::default())
                            .await
                    } else {
                        self.engines
                            .groq_models(&key, &Cancellation::default())
                            .await
                    }
                })?;
                let settings = self.inner.lock().settings.clone();
                let selected = settings
                    .engine(&settings.speech_engine)
                    .map_err(|e| e.to_string())?;
                let speech = if selected.provider == provider {
                    selected
                } else {
                    settings
                        .engine(&format!("{prefix}.speech"))
                        .map_err(|e| e.to_string())?
                };
                let text = settings
                    .engine(&format!("{prefix}.text"))
                    .map_err(|e| e.to_string())?;
                return Ok(
                    json!({"speech_model": speech.model, "speech_available": available.contains(&speech.model),
                    "text_model": text.model, "text_available": available.contains(&text.model)}),
                );
            }
            "key_status" => return Ok(json!(platform::has_secret(id()?)?)),
            "set_key" => {
                platform::set_secret(id()?, p["key"].as_str().unwrap_or(""))?;
            }
            "delete_key" => platform::delete_secret(id()?)?,
            "dismiss_message" => {
                self.inner.lock().message = None;
            }
            _ => return Err("Ukendt handling.".into()),
        }
        self.emit();
        Ok(Value::Null)
    }
}
fn result(a: &Active, original: String, processed: Option<String>, fallback: bool) -> Dictation {
    Dictation {
        id: uuid::Uuid::new_v4().to_string(),
        generation: a.session.generation.clone(),
        origin: a.session.origin.clone(),
        created_at: a.session.created_at,
        original,
        processed,
        fallback,
        delivery: Delivery::Pending,
        plan: a.session.plan.clone(),
        duration_secs: a.duration,
    }
}
pub fn key_id(e: &EngineConfig) -> String {
    match e.provider {
        Provider::Custom | Provider::Ollama => e.id.clone(),
        p => serde_json::to_value(p)
            .unwrap_or(Value::Null)
            .as_str()
            .unwrap_or("none")
            .to_string(),
    }
}
pub(crate) fn engine_secret(engine: &EngineConfig) -> Result<Option<String>> {
    if matches!(
        engine.provider,
        Provider::LocalWhisper
            | Provider::LocalParakeet
            | Provider::LocalText
            | Provider::ClaudeCli
            | Provider::CodexCli
            | Provider::GeminiCli
    ) {
        Ok(None)
    } else {
        platform::secret(&key_id(engine))
    }
}
fn needs_key(p: Provider) -> bool {
    matches!(
        p,
        Provider::OpenAi
            | Provider::Groq
            | Provider::ElevenLabs
            | Provider::Anthropic
            | Provider::Gemini
    )
}
