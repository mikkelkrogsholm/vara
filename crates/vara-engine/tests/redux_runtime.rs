//! Optional real-weight acceptance; normal tests never download models or need credentials.
use std::{path::PathBuf, sync::Arc};
use vara_core::Settings;
use vara_engine::{Cancellation, Engines};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires VARA_TEST_MODELS containing verified Redux weights"]
async fn real_redux_cancellation_and_next_session() {
    let root = PathBuf::from(std::env::var_os("VARA_TEST_MODELS").expect("VARA_TEST_MODELS"));
    let engines = Arc::new(Engines::new(root).unwrap());
    assert!(engines.ready("local.parakeet"));
    let engine = Settings::default()
        .engine("local.parakeet")
        .unwrap()
        .clone();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Samples/da_test.wav");
    let first = engines
        .speech(&engine, &fixture, None, None, &Cancellation::default())
        .await
        .unwrap();
    assert!(!first.is_empty());
    // A long recording keeps real inference in flight long enough to accept cancellation.
    let scratch = tempfile::tempdir().unwrap();
    let audio = scratch.path().join("long.wav");
    let mut reader = hound::WavReader::open(&fixture).unwrap();
    let spec = reader.spec();
    let samples: Vec<i16> = reader.samples::<i16>().collect::<Result<_, _>>().unwrap();
    let mut writer = hound::WavWriter::create(&audio, spec).unwrap();
    for _ in 0..40 {
        for sample in &samples {
            writer.write_sample(*sample).unwrap();
        }
    }
    writer.finalize().unwrap();
    let cancel = Cancellation::default();
    let request_cancel = cancel.clone();
    let request_engine = engine.clone();
    let request_engines = engines.clone();
    let task = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(request_engines.speech(
            &request_engine,
            &audio,
            None,
            None,
            &request_cancel,
        ))
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(cancel.cancel());
    let error = task.await.unwrap().unwrap_err();
    assert_eq!(error, "Annulleret.");
    // The cached model is reusable; decoder/session state from the cancelled request is gone.
    let next = engines
        .speech(&engine, &fixture, None, None, &Cancellation::default())
        .await
        .unwrap();
    assert_eq!(first, next);
}
