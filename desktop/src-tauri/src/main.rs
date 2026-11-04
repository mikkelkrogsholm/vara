#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    #[cfg(all(feature = "test-harness", target_os = "macos"))]
    if std::env::var_os("VARA_TEST_NATIVE_LOCALIZATION").is_some() {
        vara_desktop::run_native_localization_fixture();
        return;
    }
    if vara_engine::who::worker_entry() {
        return;
    }
    if vara_platform::credential_worker_entry() {
        return;
    }
    if vara_desktop::local_backend::worker_entry() {
        return;
    }
    vara_desktop::run()
}
