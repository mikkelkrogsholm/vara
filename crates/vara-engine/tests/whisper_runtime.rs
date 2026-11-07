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
    let root = tempfile::tempdir().unwrap();
    for artifact in artifacts("local.whisper").unwrap() {
        assert!(verified(&source.join(artifact.name), artifact).unwrap());
        std::os::unix::fs::symlink(source.join(artifact.name), root.path().join(artifact.name))
            .unwrap();
    }
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
    for artifact in artifacts("local.whisper").unwrap() {
        std::fs::remove_file(root.path().join(artifact.name)).unwrap();
    }
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
#[ignore = "requires VARA_TEST_WHISPER_MODELS; pinned real weights, no downloads"]
async fn whisper_mixed_silent_windows_have_valid_speech_timings() {
    let (root, engines, engine) = real_engines();
    let mut reader = hound::WavReader::open(fixture("da-000.wav")).unwrap();
    let speech: Vec<_> = reader.samples::<i16>().map(Result::unwrap).collect();
    for quiet in [false, true] {
        let samples: Vec<_> = speech
            .iter()
            .map(|s| {
                if quiet {
                    (*s as f32 * 0.02).round() as i16
                } else {
                    *s
                }
            })
            .collect();
        let short = root.path().join(format!("short-{quiet}.wav"));
        wav(&short, samples.iter().copied());
        let mixed = root.path().join(format!("mixed-{quiet}.wav"));
        wav(
            &mixed,
            std::iter::repeat_n(0, 33 * 16000)
                .chain(samples)
                .chain(std::iter::repeat_n(0, 33 * 16000)),
        );
        for language in [Some("da"), None] {
            let start = Instant::now();
            let direct_control = engines
                .speech(&engine, &short, language, None, &Cancellation::default())
                .await
                .unwrap();
            let timed_control = engines
                .transcribe_file(
                    &engine,
                    &short,
                    language,
                    None,
                    &Cancellation::default(),
                    &|_| {},
                )
                .unwrap();
            let direct = engines
                .speech(&engine, &mixed, language, None, &Cancellation::default())
                .await
                .unwrap();
            let timed = engines
                .transcribe_file(
                    &engine,
                    &mixed,
                    language,
                    None,
                    &Cancellation::default(),
                    &|_| {},
                )
                .unwrap();
            emit(
                &format!("mixed/quiet={quiet}/language={language:?}"),
                start,
                serde_json::json!({"direct_control":direct_control,"timed_control":timed_control,"direct":direct,"timed":timed}),
            );
            assert_eq!(
                direct, direct_control,
                "long silence must retain full direct speech"
            );
            assert_eq!(
                timed.text, timed_control.text,
                "long silence must retain full timed speech, including quiet opening"
            );
            assert!(
                timed.text.to_lowercase().contains("europa")
                    && timed.text.to_lowercase().contains("ikke")
            );
            timed.validate(78.84).unwrap();
            assert_eq!(timed.segments.len(), timed_control.segments.len());
            for (s, control) in timed.segments.iter().zip(&timed_control.segments) {
                assert!(s.start.unwrap() >= 33. && s.end.unwrap() <= 45.84);
                assert!((s.start.unwrap() - control.start.unwrap() - 33.).abs() < 0.011);
                assert!((s.end.unwrap() - control.end.unwrap() - 33.).abs() < 0.011);
            }
        }
    }
    let mut reader = hound::WavReader::open(fixture("da-001.wav")).unwrap();
    let second: Vec<_> = reader.samples::<i16>().map(Result::unwrap).collect();
    let multiple = root.path().join("multiple.wav");
    wav(
        &multiple,
        std::iter::repeat_n(0, 33 * 16000)
            .chain(speech.clone())
            .chain(std::iter::repeat_n(0, 33 * 16000))
            .chain(second)
            .chain(std::iter::repeat_n(0, 33 * 16000)),
    );
    for language in [Some("da"), None] {
        let start = Instant::now();
        let timed = engines
            .transcribe_file(
                &engine,
                &multiple,
                language,
                None,
                &Cancellation::default(),
                &|_| {},
            )
            .unwrap();
        emit(
            &format!("multiple/language={language:?}"),
            start,
            serde_json::json!({"timed":timed}),
        );
        timed.validate(118.56).unwrap();
        assert!(timed.text.to_lowercase().find("europa").unwrap() < timed.text.find('%').unwrap());
        assert!(timed.segments.iter().any(|s| s.start.unwrap() > 33.
            && s.end.unwrap() < 45.84
            && s.text.contains("Europa")));
        assert!(
            timed.segments.iter().any(|s| s.start.unwrap() > 78.84
                && s.end.unwrap() < 85.56
                && s.text.contains('%'))
        );
        let mut ids = std::collections::HashSet::new();
        assert!(timed.segments.iter().all(|s| ids.insert(&s.id)));
    }
    let delayed = root.path().join("delayed-300.wav");
    wav(
        &delayed,
        std::iter::repeat_n(0, 300 * 16000)
            .chain(speech)
            .chain(std::iter::repeat_n(0, 33 * 16000)),
    );
    let start = Instant::now();
    let delayed_timed = engines
        .transcribe_file(
            &engine,
            &delayed,
            Some("da"),
            None,
            &Cancellation::default(),
            &|_| {},
        )
        .unwrap();
    emit(
        "empty-first-300s-chunk",
        start,
        serde_json::json!({"timed":delayed_timed}),
    );
    delayed_timed.validate(345.84).unwrap();
    assert!(
        delayed_timed.text.to_lowercase().contains("europa")
            && delayed_timed.text.to_lowercase().contains("ikke")
    );
    assert!(
        delayed_timed
            .segments
            .iter()
            .all(|s| s.start.unwrap() >= 300. && s.end.unwrap() <= 312.84)
    );
    let start = Instant::now();
    let boundary = engines
        .transcribe_file(
            &engine,
            &fixture("long-boundaries.wav"),
            Some("da"),
            None,
            &Cancellation::default(),
            &|_| {},
        )
        .unwrap();
    emit(
        "existing-321s-boundary-fixture",
        start,
        serde_json::json!({"timed":boundary}),
    );
    boundary.validate(321.).unwrap();
    // These are four legitimate repeated utterances, not duplicated phantom text.
    for offset in [28., 58., 148., 298.] {
        assert!(
            boundary.segments.iter().any(|s| s.end.unwrap() > offset
                && s.start.unwrap() < offset + 12.84
                && s.text.to_lowercase().contains("europa")),
            "lost speech at {offset}: {boundary:?}"
        );
    }
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires VARA_TEST_WHISPER_MODELS; pinned real weights, no downloads"]
async fn whisper_base_only_and_corrupt_vad_fail_explicitly() {
    let (valid_root, valid_engines, _) = real_engines();
    assert!(valid_engines.ready("local.whisper"));
    // Runs under denied networking: verified managed weights must not be refetched.
    vara_engine::install(
        "local.whisper",
        valid_root.path(),
        &Cancellation::default(),
        |p| assert_eq!(p.phase, "ready"),
    )
    .await
    .unwrap();
    let source = PathBuf::from(std::env::var_os("VARA_TEST_WHISPER_MODELS").unwrap());
    for corrupt in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let base = &artifacts("local.whisper").unwrap()[0];
        let vad = &artifacts("local.whisper").unwrap()[1];
        std::os::unix::fs::symlink(source.join(base.name), root.path().join(base.name)).unwrap();
        if corrupt {
            std::fs::write(root.path().join(vad.name), vec![0u8; vad.bytes as usize]).unwrap();
        }
        let engines = Engines::new(root.path().into()).unwrap();
        assert!(!engines.ready("local.whisper"));
        let engine = vara_core::Settings::default()
            .engine("local.whisper")
            .unwrap()
            .clone();
        let error = engines
            .speech(
                &engine,
                &fixture("da-000.wav"),
                Some("da"),
                None,
                &Cancellation::default(),
            )
            .await
            .unwrap_err();
        assert_eq!(
            vara_core::UserMessage::decode(&error).unwrap().code,
            "whisper.load"
        );
    }
}
