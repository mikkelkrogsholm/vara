//! Explicit opt-in pinned Whisper installer into a disposable directory.
use vara_engine::{Cancellation, install};
#[tokio::main]
async fn main() {
    let root = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("temporary model directory"),
    );
    let complete_existing = std::env::args().any(|arg| arg == "--complete-existing");
    assert!(
        !root.exists() || complete_existing,
        "use a new task-owned directory or explicit --complete-existing"
    );
    let cancel = Cancellation::default();
    let interrupt = std::env::args().any(|arg| arg == "--cancel-after-progress");
    let result = install("local.whisper", &root, &cancel, |progress| {
        if interrupt && progress.phase == "downloading" {
            println!("cancelling after {} bytes", progress.received);
            cancel.cancel();
        }
    })
    .await;
    if interrupt {
        assert_eq!(
            result.unwrap_err(),
            vara_core::UserMessage::new("runtime.cancelled").wire()
        );
        assert!(!root.join("ggml-base.bin").exists());
        assert!(!root.join("ggml-base.bin.part").exists());
        println!("interrupted download left no final or partial model");
        return;
    }
    result.expect("pinned SHA256-verified Whisper Base and VAD");
    assert!(
        vara_engine::Engines::new(root.clone())
            .unwrap()
            .ready("local.whisper")
    );
    println!(
        "verified Whisper Base + VAD; ready=true at {}",
        root.display()
    );
}
