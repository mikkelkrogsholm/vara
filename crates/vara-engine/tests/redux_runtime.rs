//! Explicit real Engines acceptance. Normal tests neither download nor require weights.
//! Run serially under an external timeout and a process-local network-deny sandbox.
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use vara_core::{Settings, TextPlan, Timing};
use vara_engine::{Cancellation, Engines, artifacts, model_directory, verified};
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../Samples/acceptance/audio")
        .join(name)
}
fn record(id: &str, phase: &str, start: Instant, result: serde_json::Value) {
    println!(
        "{}",
        serde_json::json!({"engine":id,"phase":phase,"elapsed_s":start.elapsed().as_secs_f64(),"result":result})
    );
}
#[cfg(unix)]
fn linked_models(source: &Path, id: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let dest = model_directory(root.path(), id);
    std::fs::create_dir_all(&dest).unwrap();
    for a in artifacts(id).unwrap() {
        let path = model_directory(source, id).join(a.name);
        assert!(verified(&path, a).unwrap(), "unverified {}", path.display());
        println!(
            "{}",
            serde_json::json!({"engine":id,"artifact":a.name,"sha256":a.sha256,"bytes":a.bytes})
        );
        std::os::unix::fs::symlink(path, dest.join(a.name)).unwrap();
    }
    root
}
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires VARA_TEST_MODELS and VARA_TEST_WHISPER_MODELS; real weights, no downloads"]
async fn real_local_runtime_acceptance() {
    let installed = PathBuf::from(std::env::var_os("VARA_TEST_MODELS").expect("VARA_TEST_MODELS"));
    let whisper = PathBuf::from(
        std::env::var_os("VARA_TEST_WHISPER_MODELS").expect("VARA_TEST_WHISPER_MODELS"),
    );
    let settings = Settings::default();
    let mut parakeet_text = String::new();
    let mut quality_failures = Vec::new();
    for (id, source) in [
        ("local.parakeet", installed.as_path()),
        ("local.whisper", whisper.as_path()),
    ] {
        let root = linked_models(source, id);
        let engines = Arc::new(Engines::new(root.path().into()).unwrap());
        let engine = settings.engine(id).unwrap().clone();
        assert!(engines.ready(id));
        let audio = fixture("da-000.wav");
        let start = Instant::now();
        let first = engines
            .speech(&engine, &audio, Some("da"), None, &Cancellation::default())
            .await
            .unwrap();
        assert!(!first.trim().is_empty());
        record(id, "cold", start, serde_json::json!({"text":first}));
        if id == "local.parakeet" {
            parakeet_text = first.clone();
        }
        let start = Instant::now();
        let transcript = engines
            .transcribe_file(
                &engine,
                &audio,
                Some("da"),
                None,
                &Cancellation::default(),
                &|_| {},
            )
            .unwrap();
        transcript
            .validate(vara_engine::audio_duration(&audio).unwrap())
            .unwrap();
        assert!(!transcript.segments.is_empty());
        let words: usize = transcript.segments.iter().map(|s| s.words.len()).sum();
        if id == "local.whisper" {
            assert_eq!(transcript.timing, Timing::Segment);
            assert_eq!(words, 0);
        } else {
            assert_eq!(transcript.timing, Timing::Word);
            assert!(words > 0);
        }
        record(
            id,
            "warm_timed_file",
            start,
            serde_json::to_value(&transcript).unwrap(),
        );
        let cancel = Cancellation::default();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let worker = engines.clone();
        let request = engine.clone();
        let token = cancel.clone();
        let start = Instant::now();
        let task = tokio::task::spawn_blocking(move || {
            tx.send(()).unwrap();
            tokio::runtime::Handle::current().block_on(worker.speech(
                &request,
                &fixture("long-boundaries.wav"),
                Some("da"),
                None,
                &token,
            ))
        });
        rx.await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!task.is_finished(), "must cancel an in-flight request");
        assert!(cancel.cancel());
        let error = task.await.unwrap().unwrap_err();
        assert_eq!(
            error,
            vara_core::UserMessage::new("runtime.cancelled").wire()
        );
        record(
            id,
            "in_flight_cancel",
            start,
            serde_json::json!({"error":error}),
        );
        // Only our links disappear: successful subsequent inference proves cached reuse.
        for a in artifacts(id).unwrap() {
            std::fs::remove_file(model_directory(root.path(), id).join(a.name)).unwrap();
        }
        assert!(!engines.ready(id));
        let start = Instant::now();
        let next = engines
            .speech(&engine, &audio, Some("da"), None, &Cancellation::default())
            .await
            .unwrap();
        assert_eq!(first, next, "no stale decoder state after cancellation");
        record(
            id,
            "cached_after_cancel_without_model_links",
            start,
            serde_json::json!({"text":next}),
        );
        for name in ["silence.wav", "noise.wav"] {
            let start = Instant::now();
            let result = engines
                .speech(
                    &engine,
                    &fixture(name),
                    Some("da"),
                    None,
                    &Cancellation::default(),
                )
                .await;
            if let Ok(text) = &result {
                quality_failures.push(format!("{id}/{name}: {text}"));
            } else {
                assert_eq!(
                    vara_core::UserMessage::decode(result.as_ref().unwrap_err())
                        .unwrap()
                        .code,
                    "engine.no_speech"
                );
            }
            record(id, name, start, serde_json::json!({"outcome":result}));
        }
    }
    let id = "local.text";
    let root = linked_models(&installed, id);
    let engines = Arc::new(Engines::new(root.path().into()).unwrap());
    let plan = TextPlan {
        engine: settings.engine(id).unwrap().clone(),
        system_prompt: "Ret kun tegnsætningen. Returner kun den rettede tekst.".into(),
        app_specific: false,
    };
    let start = Instant::now();
    let first = engines
        .text(&plan, &parakeet_text, None, &Cancellation::default())
        .await
        .unwrap();
    record(
        id,
        "cold_parakeet_to_embedded_qwen",
        start,
        serde_json::json!({"text":first}),
    );
    let cancel = Cancellation::default();
    let worker = engines.clone();
    let request = plan.clone();
    let token = cancel.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let start = Instant::now();
    let task = tokio::task::spawn_blocking(move || {
        tx.send(()).unwrap();
        tokio::runtime::Handle::current().block_on(worker.text(
            &request,
            &"dette er en lang tekst ".repeat(100),
            None,
            &token,
        ))
    });
    rx.await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!task.is_finished());
    assert!(cancel.cancel());
    assert_eq!(
        task.await.unwrap().unwrap_err(),
        vara_core::UserMessage::new("runtime.cancelled").wire()
    );
    record(
        id,
        "in_flight_cancel",
        start,
        serde_json::json!({"cancelled":true}),
    );
    for a in artifacts(id).unwrap() {
        std::fs::remove_file(root.path().join(a.name)).unwrap();
    }
    let start = Instant::now();
    let next = engines
        .text(&plan, &parakeet_text, None, &Cancellation::default())
        .await
        .unwrap();
    assert_eq!(first, next, "Qwen KV cache resets after cancellation");
    record(
        id,
        "cached_after_cancel_without_model_links",
        start,
        serde_json::json!({"text":next}),
    );
    assert!(
        quality_failures.is_empty(),
        "QUALITY FAILURE: phantom dictation must never be accepted for persistence: {quality_failures:?}"
    );
}
