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
    if std::env::args_os().nth(1).is_some_and(|arg| arg == "cli") {
        std::process::exit(vara_desktop::agent_cli::entry());
    }
    #[cfg(all(feature = "test-harness", target_os = "macos"))]
    if vara_desktop::run_native_file_fixture() {
        return;
    }
    vara_desktop::run()
}
