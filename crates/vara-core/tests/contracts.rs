use vara_core::*;
fn origin() -> Destination {
    Destination {
        app_id: "test.editor".into(),
        app_name: "Editor".into(),
        target_token: Some("opaque-field".into()),
        capture_diagnostic: None,
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
        insertion_diagnostic: None,
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
fn english_defaults_preserve_saved_preferences() {
    let defaults = Settings::default();
    assert!(
        defaults
            .general_prompt
            .contains("Preserve the input language")
    );
    for (id, label) in [
        ("local.parakeet", "Parakeet Redux · on device"),
        ("local.whisper", "Whisper · on device"),
        ("local.text", "Qwen · on device"),
        ("openai.speech", "OpenAI · speech"),
        ("elevenlabs.speech", "ElevenLabs · speech"),
        ("openai.text", "OpenAI · text"),
        ("anthropic.text", "Anthropic · text"),
        ("gemini.text", "Gemini · text"),
        ("claude.cli", "Claude Code · own login"),
        ("codex.cli", "Codex · own login"),
        ("gemini.cli", "Gemini CLI · own login"),
        ("ollama.text", "Ollama · local service"),
    ] {
        assert_eq!(
            defaults.engines.iter().find(|e| e.id == id).unwrap().label,
            label
        );
    }
    let db = Store::open(std::path::Path::new(":memory:")).unwrap();
    let mut saved = defaults;
    saved.general_prompt = "Min egen danske instruktion".into();
    saved
        .engines
        .iter_mut()
        .find(|e| e.id == "local.parakeet")
        .unwrap()
        .label = "Parakeet Redux · på enheden".into();
    saved
        .engines
        .iter_mut()
        .find(|e| e.id == "ollama.text")
        .unwrap()
        .label = "My custom engine".into();
    db.save_settings(&saved).unwrap();
    let restored = db.settings().unwrap();
    assert_eq!(restored.general_prompt, saved.general_prompt);
    assert_eq!(restored.engines, saved.engines);
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

#[test]
fn gemini_speech_is_optional_cloud_and_migrates_without_replacing_choices() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.sqlite3");
    let db = Store::open(&path).unwrap();
    let mut saved = Settings::default();
    saved.engines.retain(|e| e.id != "gemini.speech");
    saved.speech_engine = "groq.speech".into();
    saved.general_engine = "gemini.text".into();
    saved.general_ai_enabled = true;
    saved.app_rules.insert(
        origin().app_id.clone(),
        AppRule {
            app_id: origin().app_id,
            app_name: "Editor".into(),
            enabled: true,
            prompt: "Keep my rule".into(),
            engine_id: "gemini.text".into(),
        },
    );
    saved
        .engines
        .iter_mut()
        .find(|e| e.id == "gemini.text")
        .unwrap()
        .model = "custom-text-model".into();
    db.save_settings(&saved).unwrap();
    for mut restored in [db.settings().unwrap(), Store::read_settings(&path).unwrap()] {
        assert_eq!(restored.speech_engine, saved.speech_engine);
        assert_eq!(restored.general_engine, saved.general_engine);
        assert_eq!(
            restored.engine("gemini.text").unwrap().model,
            "custom-text-model"
        );
        let engine = restored.engine("gemini.speech").unwrap();
        assert_eq!(engine.kind, EngineKind::Speech);
        assert_eq!(engine.provider, Provider::Gemini);
        assert_eq!(engine.model, "gemini-3.5-transcribe");
        assert_eq!(engine.file_timing_capability(), Some("word_requested"));
        restored.speech_engine = "gemini.speech".into();
        restored.validate().unwrap();
        let plan = Plan::resolve(&restored, &origin(), ProcessingChoice::Direct).unwrap();
        assert_eq!(plan.speech.id, "gemini.speech");
        assert!(plan.text.is_none());
        restored.strict_local = true;
        assert!(restored.validate().is_err());
        let mut bad = restored.engine("gemini.speech").unwrap().clone();
        bad.locality = Locality::OnDevice;
        assert!(bad.validate().is_err());
    }
    let mut custom = saved;
    let mut engine = Settings::default().engine("gemini.speech").unwrap().clone();
    engine.model = "custom-speech-model".into();
    custom.engines.push(engine.clone());
    db.save_settings(&custom).unwrap();
    assert_eq!(
        db.settings().unwrap().engine("gemini.speech").unwrap(),
        &engine
    );
}

#[test]
fn insertion_metadata_is_optional_and_runtime_tokens_are_not_serialized() {
    let mut value = serde_json::to_value(record("legacy-insertion", 1)).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .remove("insertion_diagnostic");
    value["origin"]
        .as_object_mut()
        .unwrap()
        .remove("capture_diagnostic");
    assert!(value["origin"].get("target_token").is_none());
    let old: Dictation = serde_json::from_value(value.clone()).unwrap();
    assert!(old.insertion_diagnostic.is_none());
    assert!(old.origin.capture_diagnostic.is_none());
    assert!(old.origin.target_token.is_none());
    // Native capture and old stored records can still deserialize a token; never reserialize it.
    value["origin"]["target_token"] = "legacy-runtime-token".into();
    let old: Dictation = serde_json::from_value(value).unwrap();
    assert_eq!(
        old.origin.target_token.as_deref(),
        Some("legacy-runtime-token")
    );
    assert!(
        serde_json::to_value(old).unwrap()["origin"]
            .get("target_token")
            .is_none()
    );
}

#[test]
fn insertion_diagnostics_survive_restart_and_final_storage_failure_without_replay() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("insertion.db");
    let mut store = Store::open(&path).unwrap();
    let record = record("insertion-once", 1);
    store.commit(&record).unwrap();
    store
        .update_insertion(&record.id, InsertionOutcome::unconfirmed())
        .unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TRIGGER fail_delivery BEFORE UPDATE ON results BEGIN SELECT RAISE(FAIL, 'fixture storage failure'); END;").unwrap();
    let attempted = InsertionOutcome {
        delivery: Delivery::Inserted,
        diagnostic: InsertionDiagnostic {
            reason: InsertionReason::Confirmed,
            method: Some(InsertionMethod::Accessibility),
            os_error: None,
        },
    };
    assert!(store.update_insertion(&record.id, attempted).is_err());
    drop(store);
    let mut store = Store::open(&path).unwrap();
    let saved = store.result(&record.id).unwrap().unwrap();
    assert_eq!(saved.delivery, Delivery::Unknown);
    assert_eq!(
        saved.insertion_diagnostic,
        Some(InsertionOutcome::unconfirmed().diagnostic)
    );
    assert!(saved.origin.target_token.is_none());
    assert_eq!(saved.original, record.original);
    assert_eq!(saved.processed, record.processed);
    // A repeated commit must return the durable Unknown result, never reset it to Pending.
    assert_eq!(store.commit(&record).unwrap().delivery, Delivery::Unknown);
    assert!(
        store
            .update_insertion("missing-result", InsertionOutcome::unconfirmed())
            .is_err()
    );
    connection
        .execute_batch("DROP TRIGGER fail_delivery;")
        .unwrap();
    let refused = InsertionOutcome {
        delivery: Delivery::ManualReady,
        diagnostic: InsertionDiagnostic {
            reason: InsertionReason::FieldUnavailable,
            method: Some(InsertionMethod::Keyboard),
            os_error: Some(-25202),
        },
    };
    store.update_insertion(&record.id, refused).unwrap();
    drop(store);
    let saved = Store::open(&path)
        .unwrap()
        .result(&record.id)
        .unwrap()
        .unwrap();
    assert_eq!(saved.delivery, refused.delivery);
    assert_eq!(saved.insertion_diagnostic, Some(refused.diagnostic));
}

#[test]
fn ignored_delivery_update_cannot_authorize_an_os_handoff() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ignored-update.db");
    let mut store = Store::open(&path).unwrap();
    let record = record("ignored-update", 1);
    store.commit(&record).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TRIGGER ignore_delivery BEFORE UPDATE ON results BEGIN SELECT RAISE(IGNORE); END;").unwrap();
    assert!(
        store
            .update_insertion(&record.id, InsertionOutcome::unconfirmed())
            .is_err()
    );
    let saved = store.result(&record.id).unwrap().unwrap();
    assert_eq!(saved.delivery, Delivery::Pending);
    assert!(saved.insertion_diagnostic.is_none());
}
