//! Opt-in real Whisper regression. Normal tests never require or fetch weights.
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use vara_engine::{Cancellation, Engines, artifacts, verified};
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../Samples/acceptance/audio")
        .join(name)
}
fn emit(phase: &str, start: Instant, result: serde_json::Value) {
    println!(
        "{}",
        serde_json::json!({"phase":phase,"elapsed_s":start.elapsed().as_secs_f64(),"result":result})
    );
}
fn wav(path: &Path, samples: impl IntoIterator<Item = i16>) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for sample in samples {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}
#[cfg(unix)]
fn real_engines() -> (tempfile::TempDir, Arc<Engines>, vara_core::EngineConfig) {
    let source = PathBuf::from(
        std::env::var_os("VARA_TEST_WHISPER_MODELS").expect("VARA_TEST_WHISPER_MODELS"),
    );
    let artifact = &artifacts("local.whisper").unwrap()[0];
    assert!(verified(&source.join(artifact.name), artifact).unwrap());
    let root = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(source.join(artifact.name), root.path().join(artifact.name))
        .unwrap();
    let engines = Arc::new(Engines::new(root.path().into()).unwrap());
    let engine = vara_core::Settings::default()
        .engine("local.whisper")
        .unwrap()
        .clone();
    (root, engines, engine)
}
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires VARA_TEST_WHISPER_MODELS; pinned real weights, no downloads"]
async fn whisper_rejects_non_speech_preserves_quiet_and_cached_cancellation() {
    let (root, engines, engine) = real_engines();
    let artifact = &artifacts("local.whisper").unwrap()[0];
    for language in [Some("da"), None] {
        for name in ["silence.wav", "noise.wav"] {
            let start = Instant::now();
            let direct = engines
                .speech(
                    &engine,
                    &fixture(name),
                    language,
                    None,
                    &Cancellation::default(),
                )
                .await
                .unwrap_err();
            assert_eq!(
                direct,
                vara_core::UserMessage::new("engine.no_speech").wire()
            );
            let timed = engines
                .transcribe_file(
                    &engine,
                    &fixture(name),
                    language,
                    None,
                    &Cancellation::default(),
                    &|_| {},
                )
                .unwrap_err();
            assert_eq!(
                timed,
                vara_core::UserMessage::new("speech.file_empty").wire()
            );
            emit(
                &format!("{name}/language={language:?}"),
                start,
                serde_json::json!({"direct_error":direct,"timed_error":timed}),
            );
        }
    }
    for name in ["da-000.wav", "da-001.wav", "da-002.wav"] {
        for quiet in [false, true] {
            let audio = if quiet {
                let mut reader = hound::WavReader::open(fixture(name)).unwrap();
                let path = root.path().join(format!("quiet-{name}"));
                wav(
                    &path,
                    reader
                        .samples::<i16>()
                        .map(|s| (s.unwrap() as f32 * 0.02).round() as i16),
                );
                path
            } else {
                fixture(name)
            };
            let start = Instant::now();
            let direct = engines
                .speech(&engine, &audio, Some("da"), None, &Cancellation::default())
                .await
                .unwrap();
            let timed = engines
                .transcribe_file(
                    &engine,
                    &audio,
                    Some("da"),
                    None,
                    &Cancellation::default(),
                    &|_| {},
                )
                .unwrap();
            assert!(!direct.trim().is_empty());
            assert!(!timed.text.trim().is_empty() && !timed.segments.is_empty());
            timed
                .validate(vara_engine::audio_duration(&audio).unwrap())
                .unwrap();
            emit(
                &format!("{name}/quiet={quiet}"),
                start,
                serde_json::json!({"direct":direct,"timed":timed}),
            );
        }
    }
    let start = Instant::now();
    let automatic = engines
        .speech(
            &engine,
            &fixture("da-000.wav"),
            None,
            None,
            &Cancellation::default(),
        )
        .await
        .unwrap();
    let automatic_timed = engines
        .transcribe_file(
            &engine,
            &fixture("da-000.wav"),
            None,
            None,
            &Cancellation::default(),
            &|_| {},
        )
        .unwrap();
    assert!(!automatic.trim().is_empty() && !automatic_timed.text.trim().is_empty());
    automatic_timed
        .validate(vara_engine::audio_duration(&fixture("da-000.wav")).unwrap())
        .unwrap();
    emit(
        "da-000.wav/automatic-language",
        start,
        serde_json::json!({"direct":automatic,"timed":automatic_timed}),
    );
    let expected = engines
        .speech(
            &engine,
            &fixture("da-000.wav"),
            Some("da"),
            None,
            &Cancellation::default(),
        )
        .await
        .unwrap();
    let cancel = Cancellation::default();
    let token = cancel.clone();
    let worker = engines.clone();
    let request = engine.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
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
    assert!(!task.is_finished());
    assert!(cancel.cancel());
    assert_eq!(
        task.await.unwrap().unwrap_err(),
        vara_core::UserMessage::new("runtime.cancelled").wire()
    );
    std::fs::remove_file(root.path().join(artifact.name)).unwrap();
    assert!(!engines.ready("local.whisper"));
    let next = engines
        .speech(
            &engine,
            &fixture("da-000.wav"),
            Some("da"),
            None,
            &Cancellation::default(),
        )
        .await
        .unwrap();
    assert_eq!(expected, next);
    emit(
        "cancel-and-cached-reuse-without-model-link",
        start,
        serde_json::json!({"text":next,"cancelled":true}),
    );
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "known Base mixed-window content failure; requires VARA_TEST_WHISPER_MODELS"]
async fn whisper_mixed_silent_windows_have_valid_speech_timings() {
    let (root, engines, engine) = real_engines();
    let mixed = root.path().join("later-speech.wav");
    let mut reader = hound::WavReader::open(fixture("da-000.wav")).unwrap();
    let speech: Vec<_> = reader.samples::<i16>().map(Result::unwrap).collect();
    wav(
        &mixed,
        std::iter::repeat_n(0, 33 * 16000)
            .chain(speech)
            .chain(std::iter::repeat_n(0, 33 * 16000)),
    );
    let start = Instant::now();
    let direct = engines
        .speech(&engine, &mixed, Some("da"), None, &Cancellation::default())
        .await
        .unwrap();
    let timed = engines
        .transcribe_file(
            &engine,
            &mixed,
            Some("da"),
            None,
            &Cancellation::default(),
            &|_| {},
        )
        .unwrap();
    assert!(!direct.trim().is_empty() && !timed.text.trim().is_empty());
    emit(
        "leading-and-trailing-33s-silence-observed",
        start,
        serde_json::json!({"duration":vara_engine::audio_duration(&mixed).unwrap(),"direct":direct,"timed":timed}),
    );
    let mixed_valid = timed.validate(vara_engine::audio_duration(&mixed).unwrap());
    let mixed_bounds = timed
        .segments
        .iter()
        .all(|s| s.end.unwrap() > 30. && s.start.unwrap() < 60.);
    emit(
        "leading-and-trailing-33s-silence",
        start,
        serde_json::json!({"duration":vara_engine::audio_duration(&mixed).unwrap(),"direct":direct,"timed":timed}),
    );
    mixed_valid.unwrap();
    assert!(mixed_bounds, "phantom segment in silent window: {timed:?}");
    // Content anchor from the public da-000 reference. This is an acceptance
    // assertion, never a phrase filter in the recognition/publishing path.
    assert!(
        timed.text.to_lowercase().contains("europa"),
        "mixed-window transcript lost the spoken content: {timed:?}"
    );
}
