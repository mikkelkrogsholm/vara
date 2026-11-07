//! Offline reproducibility guard for the native speech-signal dependency repair.
use sha2::{Digest, Sha256};
#[test]
fn vendored_whisper_matches_reviewed_provenance() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("whisper-rs-sys-provenance.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["version"], "0.15.0");
    assert_eq!(
        manifest["crate_sha256"],
        "6986c0fe081241d391f09b9a071fbcbb59720c3563628c3c829057cf69f2a56f"
    );
    for (path, expected) in manifest["files"].as_object().unwrap() {
        let bytes = std::fs::read(root.join("whisper-rs-sys").join(path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            expected.as_str().unwrap(),
            "vendored source drift: {path}"
        );
    }
    let patch = std::fs::read(root.join("whisper-sot-logits.patch")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(patch)),
        manifest["patch_sha256"].as_str().unwrap()
    );
    assert!(root.join("whisper-rs-sys/whisper.cpp/LICENSE").is_file());
    for (file, key) in [("whisper-build-sync.patch", "build_patch_sha256")] {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(file)).unwrap())
            ),
            manifest[key].as_str().unwrap()
        );
    }
}
