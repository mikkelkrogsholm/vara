use vara_engine::*;
#[test]
fn corrupt_file_is_never_verified() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("model");
    std::fs::write(&path, b"bad").unwrap();
    let artifact = Artifact {
        name: "model",
        url: "https://example.invalid",
        bytes: 3,
        sha256: "wrong",
    };
    assert!(!verified(&path, &artifact).unwrap());
}
#[tokio::test]
async fn cancellation_does_not_download_or_publish() {
    let dir = tempfile::tempdir().unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(
        install("local.whisper", dir.path(), &cancel, |_| panic!(
            "must not publish"
        ))
        .await
        .is_err()
    );
    assert!(!dir.path().join("ggml-base.bin").exists());
}

#[test]
fn accepted_cancellation_wins_publication() {
    let cancel = Cancellation::default();
    assert!(cancel.cancel());
    assert!(cancel.publish(|| Ok(())).is_err());
}
#[test]
fn completed_publication_is_not_reported_as_cancelled() {
    let cancel = Cancellation::default();
    cancel.publish(|| Ok(())).unwrap();
    assert!(!cancel.cancel());
}

#[tokio::test]
async fn redux_cancellation_skips_loading_and_network() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engines::new(dir.path().into()).unwrap();
    let settings = vara_core::Settings::default();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        engine
            .speech(
                settings.engine("local.parakeet").unwrap(),
                &dir.path().join("missing.wav"),
                None,
                None,
                &cancel
            )
            .await
            .unwrap_err(),
        vara_core::UserMessage::new("runtime.cancelled").wire()
    );
    assert!(
        install("local.parakeet", dir.path(), &cancel, |_| panic!(
            "cancelled download must not publish"
        ))
        .await
        .is_err()
    );
    assert!(!engine.ready("local.parakeet"));
}
#[test]
fn redux_and_qwen_tokenizers_have_separate_namespaces() {
    let root = std::path::Path::new("models");
    let redux = model_directory(root, "local.parakeet");
    let qwen = model_directory(root, "local.text");
    assert_ne!(redux.join("tokenizer.json"), qwen.join("tokenizer.json"));
    assert_eq!(artifacts("local.parakeet").unwrap().len(), 4);
    assert_eq!(
        artifacts("local.parakeet")
            .unwrap()
            .iter()
            .map(|a| a.bytes)
            .sum::<u64>(),
        179005408
    );
}
#[tokio::test]
async fn unavailable_redux_returns_error_without_switching_provider() {
    let root = tempfile::tempdir().unwrap();
    let engines = Engines::new(root.path().into()).unwrap();
    let settings = vara_core::Settings::default();
    let audio = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Samples/da_test.wav");
    let error = engines
        .speech(
            settings.engine("local.parakeet").unwrap(),
            &audio,
            None,
            None,
            &Cancellation::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(
        vara_core::UserMessage::decode(&error).unwrap().code,
        "parakeet.load"
    );
}

#[test]
fn same_size_corruption_is_rejected_by_hash() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("model");
    std::fs::write(&path, b"bad").unwrap();
    let artifact = Artifact {
        name: "model",
        url: "https://example.invalid",
        bytes: 3,
        sha256: "770e607624d689265ca6c44884d0807d9b054d23c473c106c72be9de08b7376c8",
    };
    assert!(!verified(&path, &artifact).unwrap());
}

#[tokio::test]
async fn interrupted_install_cancellation_preserves_existing_files_and_cleans_parts() {
    for id in ["local.parakeet", "local.whisper", "local.text"] {
        let root = tempfile::tempdir().unwrap();
        let directory = model_directory(root.path(), id);
        std::fs::create_dir_all(&directory).unwrap();
        for a in artifacts(id).unwrap() {
            std::fs::write(directory.join(a.name), b"existing damaged file").unwrap();
            std::fs::write(directory.join(format!("{}.part", a.name)), b"interrupted").unwrap();
        }
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            install(id, root.path(), &cancel, |_| panic!("no ready publication"))
                .await
                .unwrap_err(),
            vara_core::UserMessage::new("runtime.cancelled").wire()
        );
        for a in artifacts(id).unwrap() {
            assert_eq!(
                std::fs::read(directory.join(a.name)).unwrap(),
                b"existing damaged file"
            );
            assert!(!directory.join(format!("{}.part", a.name)).exists());
        }
    }
}

#[tokio::test]
async fn missing_and_corrupt_local_models_return_explicit_errors_without_fallback() {
    let settings = vara_core::Settings::default();
    let audio = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../Samples/acceptance/audio/da-000.wav");
    for corrupt in [false, true] {
        for (id, expected) in [
            ("local.parakeet", "parakeet.load"),
            ("local.whisper", "whisper.load"),
            (
                "local.text",
                if corrupt {
                    "local.model_corrupt"
                } else {
                    "local.model_missing"
                },
            ),
        ] {
            let root = tempfile::tempdir().unwrap();
            if corrupt {
                let directory = model_directory(root.path(), id);
                std::fs::create_dir_all(&directory).unwrap();
                for a in artifacts(id).unwrap() {
                    std::fs::write(directory.join(a.name), b"not a model").unwrap();
                }
            }
            let engines = Engines::new(root.path().into()).unwrap();
            assert!(!engines.ready(id));
            let engine = settings.engine(id).unwrap().clone();
            let result = if id == "local.text" {
                engines
                    .text(
                        &vara_core::TextPlan {
                            engine,
                            system_prompt: "Fix punctuation".into(),
                            app_specific: false,
                        },
                        "test",
                        None,
                        &Cancellation::default(),
                    )
                    .await
            } else {
                engines
                    .speech(&engine, &audio, Some("da"), None, &Cancellation::default())
                    .await
            };
            assert_eq!(
                vara_core::UserMessage::decode(&result.unwrap_err())
                    .unwrap()
                    .code,
                expected,
                "{id} corrupt={corrupt}"
            );
        }
    }
}
