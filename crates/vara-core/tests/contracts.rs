use vara_core::*;
fn origin() -> Destination {
    Destination {
        app_id: "test.editor".into(),
        app_name: "Editor".into(),
        target_token: Some("opaque-field".into()),
    }
}
fn record(generation: &str, at: i64) -> Dictation {
    Dictation {
        id: uuid::Uuid::new_v4().to_string(),
        generation: generation.into(),
        origin: origin(),
        created_at: at,
        original: "Send ikke pakken den 12.".into(),
        processed: Some("Send ikke pakken den 12.".into()),
        fallback: false,
        delivery: Delivery::Pending,
        plan: Plan::resolve(&Settings::default(), &origin(), ProcessingChoice::Default).unwrap(),
        duration_secs: 4.,
    }
}
#[test]
fn defaults_are_local_direct_empty() {
    let s = Settings::default();
    s.validate().unwrap();
    assert_eq!(s.speech_engine, "local.parakeet");
    let p = Plan::resolve(&s, &origin(), ProcessingChoice::Default).unwrap();
    assert_eq!(p.speech.locality, Locality::OnDevice);
    assert!(p.text.is_none());
    let db = Store::open(std::path::Path::new(":memory:")).unwrap();
    assert!(db.history(None, None).unwrap().is_empty());
    assert!(db.recoveries().unwrap().is_empty());
    assert_eq!(s.retention_days, None);
}
#[test]
fn app_rule_and_model_are_frozen_at_start() {
    let mut s = Settings::default();
    s.app_rules.insert(
        origin().app_id.clone(),
        AppRule {
            app_id: origin().app_id,
            app_name: "Editor".into(),
            enabled: false,
            prompt: "En prompt aktiverer ikke AI.".into(),
            engine_id: "local.text".into(),
        },
    );
    assert!(
        Plan::resolve(&s, &origin(), ProcessingChoice::Default)
            .unwrap()
            .text
            .is_none()
    );
    s.app_rules.get_mut(&origin().app_id).unwrap().enabled = true;
    let plan = Plan::resolve(&s, &origin(), ProcessingChoice::Default).unwrap();
    s.app_rules.get_mut(&origin().app_id).unwrap().prompt = "Ny prompt".into();
    s.engines
        .iter_mut()
        .find(|e| e.id == "local.text")
        .unwrap()
        .model = "ny model".into();
    assert_ne!(plan.text.as_ref().unwrap().system_prompt, "Ny prompt");
    assert_eq!(plan.text.as_ref().unwrap().engine.model, "qwen3-0.6b");
    assert!(
        Plan::resolve(&s, &origin(), ProcessingChoice::Direct)
            .unwrap()
            .text
            .is_none()
    );
}
#[test]
fn strict_local_cannot_be_bypassed_by_custom_label() {
    let mut s = Settings {
        strict_local: true,
        ..Settings::default()
    };
    let e = s
        .engines
        .iter_mut()
        .find(|e| e.provider == Provider::Ollama)
        .unwrap();
    e.endpoint = Some("https://example.com/v1".into());
    assert!(s.validate().is_err());
    s.engines
        .iter_mut()
        .find(|e| e.provider == Provider::Ollama)
        .unwrap()
        .locality = Locality::Cloud;
    s.general_ai_enabled = true;
    s.general_engine = "ollama.text".into();
    assert!(s.validate().is_err());
    s.general_ai_enabled = false;
    assert!(Plan::resolve(&s, &origin(), ProcessingChoice::GeneralAi).is_err());
}
#[test]
fn cancellation_is_sticky_and_commit_has_a_clear_boundary() {
    let plan = Plan::resolve(&Settings::default(), &origin(), ProcessingChoice::Default).unwrap();
    let mut s = Session::new(origin(), plan.clone(), 0);
    assert_eq!(s.cancel(), CancelOutcome::Cancelled);
    assert_eq!(s.cancel(), CancelOutcome::AlreadyCancelled);
    assert!(s.advance(Stage::Recording, Stage::Recognizing).is_err());
    let mut s = Session::new(origin(), plan, 0);
    s.advance(Stage::Recording, Stage::Recognizing).unwrap();
    s.advance(Stage::Recognizing, Stage::Committing).unwrap();
    assert_eq!(s.cancel(), CancelOutcome::AlreadyCommitted);
}
#[test]
fn save_is_atomic_idempotent_and_recovers_without_replay() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("history.db");
    let mut db = Store::open(&path).unwrap();
    let r = record("one", 100);
    db.stage(&StagedText {
        generation: r.generation.clone(),
        origin: r.origin.clone(),
        plan: r.plan.clone(),
        original: r.original.clone(),
        created_at: r.created_at,
    })
    .unwrap();
    let first = db.commit(&r).unwrap();
    let mut duplicate = r.clone();
    duplicate.id = "different".into();
    duplicate.original = "Must not overwrite".into();
    assert_eq!(db.commit(&duplicate).unwrap().id, first.id);
    assert!(db.staged().unwrap().is_empty());
    assert_eq!(db.history(None, None).unwrap().len(), 1);
    assert!(db.result(&r.id).unwrap().unwrap().processed.is_some());
    drop(db);
    let db = Store::open(&path).unwrap();
    assert_eq!(
        db.result(&r.id).unwrap().unwrap().delivery,
        Delivery::ManualReady
    );
    db.update_delivery(&r.id, Delivery::Unknown).unwrap();
    drop(db);
    let db = Store::open(&path).unwrap();
    assert_eq!(
        db.result(&r.id).unwrap().unwrap().delivery,
        Delivery::Unknown
    );
}
#[test]
fn history_is_not_count_capped_and_retention_deletes_both_versions() {
    let mut db = Store::open(std::path::Path::new(":memory:")).unwrap();
    for n in 0..160 {
        db.commit(&record(&n.to_string(), n)).unwrap();
    }
    assert_eq!(db.history(None, None).unwrap().len(), 160);
    assert_eq!(db.retain(200_000, None).unwrap(), 0);
    assert_eq!(db.retain(200_000, Some(1)).unwrap(), 160);
    assert!(db.history(None, None).unwrap().is_empty());
}
#[test]
fn recovery_uses_original_expiry() {
    let plan = Plan::resolve(&Settings::default(), &origin(), ProcessingChoice::Default).unwrap();
    let mut r = Recovery {
        id: "r".into(),
        generation: "g".into(),
        origin: origin(),
        plan,
        created_at: 10,
        expires_at: 10 + AUDIO_TTL_SECS,
        file_name: "r.wav".into(),
        failed: true,
        recognized: false,
    };
    assert!(r.eligible(11));
    assert!(!r.eligible(10 + AUDIO_TTL_SECS));
    let db = Store::open(std::path::Path::new(":memory:")).unwrap();
    db.put_recovery(&r).unwrap();
    r.generation = "retry".into();
    db.put_recovery(&r).unwrap();
    assert_eq!(db.recovery("r").unwrap().unwrap().expires_at, 86410);
    db.delete_recovery("r").unwrap();
    assert!(db.recoveries().unwrap().is_empty());
}
#[test]
fn disabled_prompts_do_not_enable_processing() {
    let mut s = Settings::default();
    s.general_prompt = "Gør noget andet".into();
    assert!(
        Plan::resolve(&s, &origin(), ProcessingChoice::Default)
            .unwrap()
            .text
            .is_none()
    );
    assert!(
        Plan::resolve(&s, &origin(), ProcessingChoice::GeneralAi)
            .unwrap()
            .text
            .is_some()
    );
}

#[test]
fn cancelled_generation_cannot_reappear_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let mut store = Store::open(&path).unwrap();
    let r = record("cancelled-generation", 100);
    let stage = StagedText {
        generation: r.generation.clone(),
        origin: r.origin.clone(),
        plan: r.plan.clone(),
        original: r.original.clone(),
        created_at: r.created_at,
    };
    store.stage(&stage).unwrap();
    store.cancel_generation(&r.generation).unwrap();
    drop(store);
    let mut store = Store::open(&path).unwrap();
    assert!(store.staged().unwrap().is_empty());
    assert!(matches!(store.stage(&stage), Err(Error::Cancelled)));
    assert!(matches!(store.commit(&r), Err(Error::Cancelled)));
    assert!(store.history(None, None).unwrap().is_empty());
}

#[test]
fn older_settings_gain_redux_without_replacing_selected_whisper() {
    let store = Store::open(std::path::Path::new(":memory:")).unwrap();
    let mut settings = Settings::default();
    settings.speech_engine = "local.whisper".into();
    settings.engines.retain(|e| e.id != "local.parakeet");
    store.save_settings(&settings).unwrap();
    let upgraded = store.settings().unwrap();
    assert_eq!(upgraded.speech_engine, "local.whisper");
    assert_eq!(
        upgraded.engine("local.parakeet").unwrap().provider,
        Provider::LocalParakeet
    );
    upgraded.validate().unwrap();
}

#[test]
fn groq_is_optional_cloud_and_strict_local_rejects_it() {
    let mut settings = Settings::default();
    assert_eq!(settings.speech_engine, "local.parakeet");
    assert_eq!(
        settings.engine("groq.speech").unwrap().model,
        "whisper-large-v3-turbo"
    );
    assert_eq!(
        settings.engine("groq.speech.accurate").unwrap().model,
        "whisper-large-v3"
    );
    settings.strict_local = true;
    settings.speech_engine = "groq.speech".into();
    assert!(Plan::resolve(&settings, &origin(), ProcessingChoice::Default).is_err());
}
