//! Isolated host acceptance; scripted inference and delivery, real Service/store/audio lifecycle.
//! No microphone, focus capture, key access, or OS insertion. Feature excluded from releases.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct Scripted {
    pub speech: Result<String>,
    pub text: Result<String>,
    pub deliveries: Mutex<Vec<(Destination, String)>>,
    pub started: AtomicBool,
    pub delay_ms: u64,
}
fn service(
    app: &AppHandle,
    root: PathBuf,
    speech: Result<String>,
    text: Result<String>,
    delay_ms: u64,
) -> Arc<Service> {
    let mut service = Service::new(app.clone(), root).unwrap();
    Arc::get_mut(&mut service).unwrap().scripted = Some(Scripted {
        speech,
        text,
        deliveries: Mutex::new(vec![]),
        started: AtomicBool::new(false),
        delay_ms,
    });
    service
}
fn active(service: &Arc<Service>, ai: bool) -> Active {
    let origin = Destination {
        app_id: "fixture.origin".into(),
        app_name: "Origin fixture".into(),
        target_token: None,
    };
    let mut settings = Settings::default();
    settings.speech_engine = "gemini.speech".into();
    settings.general_engine = "gemini.text".into();
    settings.general_ai_enabled = ai;
    let mut session = Session::new(
        origin,
        Plan::resolve(
            &settings,
            &Destination {
                app_id: "fixture.origin".into(),
                app_name: "Origin fixture".into(),
                target_token: None,
            },
            ProcessingChoice::Default,
        )
        .unwrap(),
        now(),
    );
    session.stage = Stage::Recognizing;
    let path = service.audio.join(format!("{}.wav", session.recovery_id));
    std::fs::write(&path, b"task owned audio lifecycle fixture").unwrap();
    let recovery = Recovery {
        id: session.recovery_id.clone(),
        generation: session.generation.clone(),
        origin: session.origin.clone(),
        plan: session.plan.clone(),
        created_at: session.created_at,
        expires_at: session.created_at + AUDIO_TTL_SECS,
        file_name: path.file_name().unwrap().to_string_lossy().into(),
        failed: false,
        recognized: false,
    };
    let a = Active {
        session,
        path,
        duration: 1.0,
        cancel: Cancellation::default(),
    };
    let mut i = service.inner.lock();
    i.store.put_recovery(&recovery).unwrap();
    i.active = Some(a.clone());
    a
}
fn run(app: &AppHandle, root: &std::path::Path, source: &std::path::Path) {
    // Timeout/refusal/empty results are provider errors by this boundary. Each
    // actual Service pipeline preserves original text and exposes recovery cause.
    for code in ["provider.timeout", "ai.incomplete", "ai.empty"] {
        let service = service(
            app,
            root.join(code),
            Ok("Original fixture".into()),
            Err(UserMessage::new(code).wire()),
            0,
        );
        let a = active(&service, true);
        tauri::async_runtime::block_on(service.clone().pipeline(a.clone()));
        assert!(!a.path.exists());
        let i = service.inner.lock();
        let history = i.store.history(Some("fixture.origin"), None).unwrap();
        assert_eq!(history.len(), 1);
        let doc = &history[0];
        assert_eq!(doc.original, "Original fixture");
        assert_eq!(doc.finished_text(), "Original fixture");
        assert!(doc.fallback && doc.processed.is_none());
        assert_eq!(doc.delivery, Delivery::ManualReady);
        assert!(i.store.recoveries().unwrap().is_empty());
        let message = UserMessage::decode(i.message.as_ref().unwrap()).unwrap();
        assert_eq!(message.code, "host.ai_fallback");
        assert!(i.message.as_ref().unwrap().contains(code));
        assert_eq!(
            service.scripted.as_ref().unwrap().deliveries.lock().len(),
            1
        );
    }
    let success = service(
        app,
        root.join("success"),
        Ok("original".into()),
        Ok("Processed fixture".into()),
        0,
    );
    let a = active(&success, true);
    tauri::async_runtime::block_on(success.clone().pipeline(a.clone()));
    let history = success
        .inner
        .lock()
        .store
        .history(Some("fixture.origin"), None)
        .unwrap();
    assert_eq!(history[0].finished_text(), "Processed fixture");
    assert_eq!(history[0].original, "original");
    assert!(!history[0].fallback && !a.path.exists());
    let deliveries = success.scripted.as_ref().unwrap().deliveries.lock();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].0.app_id, "fixture.origin");
    assert_eq!(deliveries[0].1, "Processed fixture");
    drop(deliveries);
    // The real engine's empty speech outcome must stop before AI, history, and
    // delivery. Retained recovery/audio follows the existing speech-error policy.
    for ai in [false, true] {
        let no_speech = service(
            app,
            root.join(format!("no-speech-ai-{ai}")),
            Err(UserMessage::new("engine.no_speech").wire()),
            Ok("must never publish".into()),
            0,
        );
        let a = active(&no_speech, ai);
        tauri::async_runtime::block_on(no_speech.clone().pipeline(a.clone()));
        let i = no_speech.inner.lock();
        assert!(i.active.is_none());
        assert!(i.store.history(None, None).unwrap().is_empty());
        assert_eq!(
            UserMessage::decode(i.message.as_ref().unwrap())
                .unwrap()
                .code,
            "engine.no_speech"
        );
        let recovery = i.store.recovery(&a.session.recovery_id).unwrap().unwrap();
        assert!(recovery.failed && !recovery.recognized && a.path.exists());
        assert!(
            no_speech
                .scripted
                .as_ref()
                .unwrap()
                .deliveries
                .lock()
                .is_empty()
        );
    }
    // A failed retry retains exactly the initial deadline. A successful retry
    // publishes once, clears audio, and forces manual delivery (no stale token).
    let mut retry = service(
        app,
        root.join("retry"),
        Err(UserMessage::new("provider.timeout").wire()),
        Ok("unused".into()),
        20,
    );
    let a = active(&retry, false);
    tauri::async_runtime::block_on(retry.clone().pipeline(a.clone()));
    let initial = retry
        .inner
        .lock()
        .store
        .recovery(&a.session.recovery_id)
        .unwrap()
        .unwrap();
    assert!(initial.failed && a.path.exists());
    retry.retry(&initial.id).unwrap();
    wait_idle(&retry);
    let retained = retry
        .inner
        .lock()
        .store
        .recovery(&initial.id)
        .unwrap()
        .unwrap();
    assert_eq!(retained.expires_at, initial.expires_at);
    assert_eq!(retained.created_at, initial.created_at);
    assert!(
        retry
            .inner
            .lock()
            .store
            .history(None, None)
            .unwrap()
            .is_empty()
    );
    Arc::get_mut(&mut retry)
        .unwrap()
        .scripted
        .as_mut()
        .unwrap()
        .speech = Ok("Retry original".into());
    retry.retry(&initial.id).unwrap();
    wait_idle(&retry);
    let i = retry.inner.lock();
    assert_eq!(i.store.history(None, None).unwrap().len(), 1);
    assert!(i.store.recoveries().unwrap().is_empty());
    assert!(!a.path.exists());
    drop(i);
    // Restart converts retained unfinished audio to an explicit failed recovery.
    let restart = service(
        app,
        root.join("restart"),
        Ok("unused".into()),
        Ok("unused".into()),
        0,
    );
    let a = active(&restart, false);
    let recovery_id = a.session.recovery_id.clone();
    drop(restart);
    let reopened = Service::new(app.clone(), root.join("restart")).unwrap();
    assert!(
        reopened
            .inner
            .lock()
            .store
            .recovery(&recovery_id)
            .unwrap()
            .unwrap()
            .failed
    );
    reopened.delete_recovery(&recovery_id).unwrap();
    assert!(!a.path.exists());
    assert!(reopened.inner.lock().store.recoveries().unwrap().is_empty());
    // Restart after recognition preserves staged original and never replays AI
    // or insertion, even when the saved plan requested AI processing.
    let staged_root = root.join("staged-restart");
    std::fs::create_dir(&staged_root).unwrap();
    let store = Store::open(&staged_root.join("vara.sqlite3")).unwrap();
    let origin = Destination {
        app_id: "fixture.origin".into(),
        app_name: "Origin fixture".into(),
        target_token: None,
    };
    let mut settings = Settings::default();
    settings.general_ai_enabled = true;
    store
        .stage(&StagedText {
            generation: "staged-fixture".into(),
            origin: origin.clone(),
            plan: Plan::resolve(&settings, &origin, ProcessingChoice::Default).unwrap(),
            original: "Recovered original".into(),
            created_at: now(),
        })
        .unwrap();
    drop(store);
    let recovered = Service::new(app.clone(), staged_root.clone()).unwrap();
    let history = recovered
        .inner
        .lock()
        .store
        .history(Some("fixture.origin"), None)
        .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].finished_text(), "Recovered original");
    assert!(history[0].fallback && history[0].processed.is_none());
    assert_eq!(history[0].delivery, Delivery::ManualReady);
    drop(recovered);
    let recovered = Service::new(app.clone(), staged_root).unwrap();
    assert_eq!(
        recovered
            .inner
            .lock()
            .store
            .history(None, None)
            .unwrap()
            .len(),
        1
    );
    // Injected boundary clock: no wait and no TTL reset.
    let expiry = service(
        app,
        root.join("expiry"),
        Err("unused".into()),
        Ok("unused".into()),
        0,
    );
    let a = active(&expiry, false);
    let end = a.session.created_at + AUDIO_TTL_SECS;
    expiry.maintenance_at(end - 1).unwrap();
    assert!(a.path.exists());
    expiry.maintenance_at(end).unwrap();
    assert!(!a.path.exists() && expiry.inner.lock().active.is_none());
    assert!(expiry.retry(&a.session.recovery_id).is_err());
    // Cancellation wins over a late successful recognizer and cannot publish.
    let cancel = service(
        app,
        root.join("cancel"),
        Ok("Late text".into()),
        Ok("unused".into()),
        100,
    );
    let a = active(&cancel, false);
    let worker = cancel.clone();
    let late = a.clone();
    let thread = std::thread::spawn(move || tauri::async_runtime::block_on(worker.pipeline(late)));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !cancel
        .scripted
        .as_ref()
        .unwrap()
        .started
        .load(Ordering::SeqCst)
    {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(cancel.cancel().unwrap(), CancelOutcome::Cancelled);
    thread.join().unwrap();
    assert!(
        cancel
            .inner
            .lock()
            .store
            .history(None, None)
            .unwrap()
            .is_empty()
    );
    assert!(
        cancel
            .scripted
            .as_ref()
            .unwrap()
            .deliveries
            .lock()
            .is_empty()
    );
    assert!(!a.path.exists());
    // Real file owner and Service can both finish under one isolated profile.
    // The owner's ASR result is explicitly injected; no provider is contacted.
    assert!(std::env::var_os("VARA_TEST_ASR_FIXTURE").is_some());
    let concurrent_root = root.join("concurrent");
    let concurrent = service(
        app,
        concurrent_root.clone(),
        Ok("Concurrent dictation".into()),
        Ok("unused".into()),
        100,
    );
    let response = crate::local_backend::call(
        &concurrent_root,
        crate::local_backend::Operation::Submit {
            request: vara_app::SubmitRequest {
                source: source.to_owned(),
                engine_id: Some("gemini.speech".into()),
                language: None,
                request_id: Some("service-concurrent-file".into()),
                diarize: false,
                speakers: None,
            },
        },
        &std::env::current_exe().unwrap(),
    )
    .unwrap();
    assert!(response.error.is_none());
    let file_id = response.result.unwrap()["job_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let a = active(&concurrent, false);
    let file = crate::local_backend::call(
        &concurrent_root,
        crate::local_backend::Operation::Status {
            id: file_id.clone(),
        },
        &std::env::current_exe().unwrap(),
    )
    .unwrap()
    .result
    .unwrap();
    assert!(
        file["stage"] == "queued" || file["stage"] == "transcribing",
        "file must still be in flight: {file}"
    );
    assert!(concurrent.inner.lock().active.is_some());
    tauri::async_runtime::block_on(concurrent.clone().pipeline(a));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        let file = crate::local_backend::call(
            &concurrent_root,
            crate::local_backend::Operation::Status {
                id: file_id.clone(),
            },
            &std::env::current_exe().unwrap(),
        )
        .unwrap()
        .result
        .unwrap();
        if file["stage"] == "ready" {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{file}");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert_eq!(
        concurrent
            .inner
            .lock()
            .store
            .history(None, None)
            .unwrap()
            .len(),
        1
    );
    println!(
        "Service acceptance passed: no-speech rejects history/delivery with AI off/on, 3 AI fallbacks, processed-first/origin, audio cleanup, retry identity/TTL/success, restart/manual-delete/staged-original, injected expiry, late-success cancellation, concurrent file/dictation; inference and OS insertion mocked"
    );
}
fn wait_idle(service: &Arc<Service>) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while service.inner.lock().active.is_some() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    // The blocking task releases its final Arc immediately after active clears.
    while Arc::strong_count(service) != 1 {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}
pub(super) fn entry() -> bool {
    if std::env::args_os().nth(1).as_deref() != Some(std::ffi::OsStr::new("--vara-test-service")) {
        return false;
    }
    let root = PathBuf::from(
        std::env::args_os()
            .nth(2)
            .expect("new isolated profile directory"),
    );
    let source = PathBuf::from(std::env::args_os().nth(3).expect("public WAV source"));
    std::fs::create_dir(&root).expect("refuse existing profile");
    let mut context = crate::app_context();
    context.config_mut().app.windows.clear();
    tauri::Builder::default()
        .setup(move |app| {
            let app = app.handle().clone();
            std::thread::spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(&app, &root, &source)
                }));
                app.exit(if result.is_ok() { 0 } else { 1 });
            });
            Ok(())
        })
        .build(context)
        .expect("service fixture app")
        .run(|_, _| {});
    true
}
