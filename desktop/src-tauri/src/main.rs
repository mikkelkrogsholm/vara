#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if vara_platform::credential_worker_entry() {
        return;
    }
    vara_desktop::run()
}
