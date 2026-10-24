mod service;
use service::Service;
use std::sync::{Arc, OnceLock, Weak};
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
static PHYSICAL_REGISTERED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static CAPTURING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static HOTKEY_EVENTS: OnceLock<std::sync::mpsc::Sender<bool>> = OnceLock::new();
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
static LIVE: OnceLock<Weak<Service>> = OnceLock::new();
extern "C" fn capture_key(key: u16, down: bool) {
    if CAPTURING.load(std::sync::atomic::Ordering::SeqCst) {
        if let (Some(app), Some(code)) = (APP.get(), vara_platform::physical_key_name(key)) {
            let _ = app.emit(
                "vara:shortcut-key",
                serde_json::json!({"code":code,"down":down}),
            );
        }
    }
}
extern "C" fn shortcut_event(down: bool) {
    if !CAPTURING.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(events) = HOTKEY_EVENTS.get() {
            let _ = events.send(down);
        }
    }
}
fn register_shortcut(app: &tauri::AppHandle, value: &str) -> Result<(), String> {
    if value.starts_with("Physical:") {
        let result = vara_platform::set_physical_shortcut(value, shortcut_event);
        PHYSICAL_REGISTERED.store(result.is_ok(), std::sync::atomic::Ordering::SeqCst);
        result
    } else {
        let shortcut: Shortcut = value.parse().map_err(|_| {
            "Tastkombinationen kunne ikke fortolkes. Vælg genvejen igen.".to_string()
        })?;
        app.global_shortcut().register(shortcut).map_err(|_| "Genvejen kunne ikke registreres; den kan være reserveret eller i brug. Den tidligere genvej er bevaret.".to_string())
    }
}
fn unregister_shortcut(app: &tauri::AppHandle, value: &str) {
    if value.starts_with("Physical:") {
        vara_platform::clear_physical_shortcut();
        PHYSICAL_REGISTERED.store(false, std::sync::atomic::Ordering::SeqCst);
    } else {
        let _ = app.global_shortcut().unregister(value);
    }
}
fn replace_shortcut(app: &tauri::AppHandle, old: &str, new: &str) -> Result<(), String> {
    unregister_shortcut(app, old);
    if let Err(error) = register_shortcut(app, new) {
        if let Err(restore) = register_shortcut(app, old) {
            return Err(format!(
                "{error} Den tidligere genvej kunne heller ikke genaktiveres: {restore}"
            ));
        }
        return Err(error);
    }
    Ok(())
}
fn run_shortcut_events(service: Weak<Service>, events: std::sync::mpsc::Receiver<bool>) {
    let mut held = false;
    for down in events {
        if CAPTURING.load(std::sync::atomic::Ordering::SeqCst) {
            held = false;
            continue;
        }
        if down == held {
            continue;
        }
        held = down;
        let Some(service) = service.upgrade() else {
            break;
        };
        let result = match (down, service.activation()) {
            (true, vara_core::Activation::Hold) if !service.is_recording() => {
                service.start(vara_core::ProcessingChoice::Default)
            }
            (false, vara_core::Activation::Hold) if service.is_recording() => service.stop(),
            (true, vara_core::Activation::Toggle) => {
                if service.is_recording() {
                    service.stop()
                } else {
                    service.start(vara_core::ProcessingChoice::Default)
                }
            }
            _ => Ok(()),
        };
        if let Err(error) = result {
            service.warn(error);
        }
    }
}
extern "C" fn hud_cancel() {
    if let Some(service) = LIVE.get().and_then(Weak::upgrade) {
        let _ = service.cancel();
    }
}
extern "C" fn hud_stop() {
    if let Some(service) = LIVE.get().and_then(Weak::upgrade) {
        if let Err(error) = service.stop() {
            service.warn(error);
        }
    }
}
extern "C" fn hud_level() -> f32 {
    LIVE.get()
        .and_then(Weak::upgrade)
        .map(|s| s.level())
        .unwrap_or(0.)
}
fn hud_callbacks() -> vara_platform::HudCallbacks {
    vara_platform::HudCallbacks {
        cancel: hud_cancel,
        stop: hud_stop,
        level: hud_level,
    }
}
#[tauri::command]
async fn command(
    state: tauri::State<'_, Arc<Service>>,
    app: tauri::AppHandle,
    action: String,
    payload: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let service = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        if action == "shortcut_capture" {
            let enabled = payload
                .as_ref()
                .and_then(|p| p["enabled"].as_bool())
                .unwrap_or(false);
            if enabled && service.is_recording() {
                return Err("Afslut dikteringen, før du vælger en genvej.".into());
            }
            if enabled {
                CAPTURING.store(true, std::sync::atomic::Ordering::SeqCst);
                match vara_platform::start_shortcut_capture(capture_key) {
                    Ok(native) => Ok(serde_json::json!({"native": native})),
                    Err(error) => {
                        CAPTURING.store(false, std::sync::atomic::Ordering::SeqCst);
                        Err(error)
                    }
                }
            } else {
                vara_platform::end_shortcut_capture();
                CAPTURING.store(false, std::sync::atomic::Ordering::SeqCst);
                Ok(serde_json::Value::Null)
            }
        } else if action == "save_settings" {
            let _settings_gate = service.settings_gate.lock();
            let settings: vara_core::Settings = serde_json::from_value(payload.unwrap_or_default())
                .map_err(|_| "Indstillingerne kunne ikke læses.")?;
            settings.validate().map_err(|e| e.to_string())?;
            let old = service.shortcut();
            let changed = old != settings.shortcut;
            if changed {
                replace_shortcut(&app, &old, &settings.shortcut)?;
            }
            if let Err(error) = service.save_settings(settings.clone()) {
                if changed {
                    if let Err(restore) = replace_shortcut(&app, &settings.shortcut, &old) {
                        return Err(format!("{error} {restore}"));
                    }
                }
                return Err(error);
            }
            Ok(serde_json::Value::Null)
        } else {
            service.command(&action, payload.unwrap_or_default())
        }
    })
    .await
    .map_err(|_| "Handlingen blev afbrudt.".to_string())?
}
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, _shortcut, event| {
                    shortcut_event(event.state() == ShortcutState::Pressed)
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![command])
        .setup(|app| {
            let _ = APP.set(app.handle().clone());
            let root = if let Some(path) = std::env::var_os("VARA_DATA_DIR") {
                std::path::PathBuf::from(path)
            } else {
                app.path().app_data_dir()?
            };
            let service =
                Service::new(app.handle().clone(), root).map_err(std::io::Error::other)?;
            let _ = LIVE.set(Arc::downgrade(&service));
            app.manage(service.clone());
            let (sender, receiver) = std::sync::mpsc::channel();
            let _ = HOTKEY_EVENTS.set(sender);
            let weak = Arc::downgrade(&service);
            std::thread::spawn(move || run_shortcut_events(weak, receiver));
            if let Err(error) = register_shortcut(app.handle(), &service.shortcut()) {
                service.warn(error);
            }
            let show = MenuItem::with_id(app, "show", "Åbn Vara", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Afslut Vara", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
                    tauri::image::Image::new_owned(vec![47, 53, 39, 255].repeat(16 * 16), 16, 16)
                }))
                .tooltip("Vara · diktering")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        if let Some(service) = app.try_state::<Arc<Service>>() {
                            let _ = service.cancel();
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;
            let maintenance = service.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    if let Err(error) = maintenance.maintenance() {
                        maintenance.warn(error);
                    }
                    maintenance.emit();
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            tauri::WindowEvent::Focused(false) => {
                CAPTURING.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            tauri::WindowEvent::Focused(true) => {
                if let Some(service) = window.try_state::<Arc<Service>>() {
                    if service.shortcut().starts_with("Physical:")
                        && !PHYSICAL_REGISTERED.load(std::sync::atomic::Ordering::SeqCst)
                    {
                        let _ = register_shortcut(window.app_handle(), &service.shortcut());
                    }
                    if let Err(error) = service.maintenance() {
                        service.warn(error);
                    }
                    service.emit();
                }
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("Vara kunne ikke starte")
        .run(|app, event| match event {
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            tauri::RunEvent::ExitRequested { .. } => {
                if let Some(service) = app.try_state::<Arc<Service>>() {
                    let _ = service.cancel();
                }
            }
            _ => {}
        });
}
