use std::path::Path;
use vara_core::UserMessage;
use vara_engine::{Cancellation, media, models, who};

#[tokio::test]
async fn origins_emit_codes_without_private_paths() {
    let cancel = Cancellation::default();
    let error = models::install("not-a-model", Path::new("/unused"), &cancel, |_| {})
        .await
        .unwrap_err();
    assert_eq!(UserMessage::decode(&error).unwrap().code, "model.unknown");
    let error = media::normalize(
        Path::new("/private/secret/does-not-exist"),
        Path::new("/unused"),
        &cancel,
    )
    .unwrap_err();
    let message = UserMessage::decode(&error).unwrap();
    assert_eq!(message.code, "media.source_open");
    assert!(!error.contains("/private/secret"));
    assert!(message.cause.is_some());
    let error = who::run_known(
        Path::new("/missing"),
        Path::new("/unused"),
        "diarize",
        None,
        Some(0),
        &cancel,
    )
    .unwrap_err();
    assert_eq!(
        UserMessage::decode(&error).unwrap().code,
        "who.speaker_count"
    );
}
