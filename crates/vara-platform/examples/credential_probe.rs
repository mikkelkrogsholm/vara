//! Opt-in disposable OS-store probe. Build release, sign, then run. No user keys.
fn main() {
    if vara_platform::credential_worker_entry() {
        return;
    }
    let id = format!("credential-probe-{}", std::process::id());
    struct Cleanup(String);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = vara_platform::delete_secret(&self.0);
        }
    }
    let _cleanup = Cleanup(id.clone());
    assert!(!vara_platform::has_secret(&id).expect("metadata before save"));
    vara_platform::set_secret(&id, "fictional-vara-probe-key").expect("save");
    assert!(vara_platform::has_secret(&id).expect("metadata after save"));
    assert_eq!(
        vara_platform::secret(&id).expect("read").as_deref(),
        Some("fictional-vara-probe-key")
    );
    vara_platform::set_secret(&id, "fictional-replacement").expect("replace");
    assert_eq!(
        vara_platform::secret(&id)
            .expect("read replacement")
            .as_deref(),
        Some("fictional-replacement")
    );
    vara_platform::delete_secret(&id).expect("delete");
    assert!(!vara_platform::has_secret(&id).expect("metadata after deletion"));
    assert!(vara_platform::secret(&id).expect("missing read").is_none());
    println!("PASS: signed Rust worker metadata, save, read, replace, delete and missing entry");
}
