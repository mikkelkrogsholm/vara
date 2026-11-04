pub mod agent_cli;
pub mod local_backend;
fn host_message(code: &str) -> String {
    vara_core::UserMessage::new(code).wire()
}
fn host_wrapped(code: &str, cause: &str) -> String {
    let mut message = vara_core::UserMessage::new(code);
    if let Some(cause) = vara_core::UserMessage::decode(cause) {
        message = message.caused_by(cause);
    }
    message.wire()
}
fn host_io(code: &str, error: &std::io::Error) -> String {
    vara_core::UserMessage::new(code)
        .caused_by(vara_core::UserMessage::io_cause(error))
        .wire()
}
fn host_json(code: &str, error: &serde_json::Error) -> String {
    vara_core::UserMessage::new(code)
        .caused_by(
            vara_core::UserMessage::new("storage.json_location")
                .number("line", error.line() as i64)
                .number("column", error.column() as i64),
        )
        .wire()
}
fn with_restore_error(error: &str, restore: &str) -> String {
    let mut message = vara_core::UserMessage::new("host.shortcut_restore");
    if let Some(mut cause) = vara_core::UserMessage::decode(error) {
        let mut tail = &mut cause;
        while let Some(ref mut next) = tail.cause {
            tail = next;
        }
        tail.cause = vara_core::UserMessage::decode(restore).map(Box::new);
        message = message.caused_by(cause);
    } else if let Some(cause) = vara_core::UserMessage::decode(restore) {
        message = message.caused_by(cause);
    }
    message.wire()
}
mod service;
mod transcription;
use service::Service;
use std::sync::{Arc, OnceLock, Weak};
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
struct NativeMenu {
    show: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
    tray: TrayIcon<tauri::Wry>,
}
impl NativeMenu {
    fn localize(&self, locale: &str) {
        let text = |code| vara_core::UserMessage::new(code).format(locale);
        let _ = self.show.set_text(text("native.menu.show"));
        let _ = self.quit.set_text(text("native.menu.quit"));
        let _ = self.tray.set_tooltip(Some(text("native.tray.tooltip")));
    }
}
fn build_native_menu(app: &tauri::App, locale: &str) -> tauri::Result<NativeMenu> {
    let show = MenuItem::with_id(
        app,
        "show",
        vara_core::UserMessage::new("native.menu.show").format(locale),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        vara_core::UserMessage::new("native.menu.quit").format(locale),
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            tauri::image::Image::new_owned(vec![47, 53, 39, 255].repeat(16 * 16), 16, 16)
        }))
        .tooltip(vara_core::UserMessage::new("native.tray.tooltip").format(locale))
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.state::<Arc<transcription::Files>>().stop();
                if let Some(service) = app.try_state::<Arc<Service>>() {
                    let _ = service.cancel();
                }
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    let native_menu = NativeMenu { show, quit, tray };
    native_menu.localize(locale);
    Ok(native_menu)
}
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
        let shortcut: Shortcut = value
            .parse()
            .map_err(|_| crate::host_message("host.shortcut_parse").to_string())?;
        app.global_shortcut()
            .register(shortcut)
            .map_err(|_| crate::host_message("host.shortcut_register").to_string())
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
            return Err(with_restore_error(&error, &restore));
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
        if action == "runtime_catalog" {
            return Ok(
                serde_json::from_str(vara_core::message::CATALOG).expect("valid runtime catalog")
            );
        }
        if action.starts_with("file_") {
            use tauri::Manager;
            let files = app.state::<Arc<transcription::Files>>();
            return files.command(&action, payload.unwrap_or_default(), &service.preferences());
        }
        if action == "shortcut_capture" {
            let enabled = payload
                .as_ref()
                .and_then(|p| p["enabled"].as_bool())
                .unwrap_or(false);
            if enabled && service.is_recording() {
                return Err(crate::host_message("host.shortcut_recording").into());
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
        } else if action == "set_ui_locale" {
            let _settings_gate = service.settings_gate.lock();
            let preference: vara_core::UiLocale = serde_json::from_value(
                payload
                    .unwrap_or_default()
                    .get("preference")
                    .cloned()
                    .unwrap_or_default(),
            )
            .map_err(|e| crate::host_json("host.locale_invalid", &e))?;
            let (locale, revision) = service.set_ui_locale(preference)?;
            if let Some(menu) = app.try_state::<NativeMenu>() {
                menu.localize(locale);
            }
            Ok(serde_json::json!({"locale": locale, "revision": revision}))
        } else if action == "save_settings" {
            let _settings_gate = service.settings_gate.lock();
            let mut settings: vara_core::Settings =
                serde_json::from_value(payload.unwrap_or_default())
                    .map_err(|e| crate::host_json("host.settings_read", &e))?;
            settings.ui_locale = service.preferences().ui_locale;
            settings.validate().map_err(|e| e.wire())?;
            let old = service.shortcut();
            let changed = old != settings.shortcut;
            if changed {
                replace_shortcut(&app, &old, &settings.shortcut)?;
            }
            if let Err(error) = service.save_settings(settings.clone()) {
                if changed {
                    if let Err(restore) = replace_shortcut(&app, &settings.shortcut, &old) {
                        return Err(with_restore_error(&error, &restore));
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
    .map_err(|_| crate::host_message("host.task_interrupted").to_string())?
}
fn app_context() -> tauri::Context<tauri::Wry> {
    tauri::generate_context!()
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
                Service::new(app.handle().clone(), root.clone()).map_err(std::io::Error::other)?;
            transcription::allow_media(app.handle(), &root)?;
            let files = transcription::Files::new(app.handle().clone(), root)
                .map_err(std::io::Error::other)?;
            app.manage(files.clone());
            #[cfg(feature = "test-harness")]
            if let Some(path) = std::env::var_os("VARA_TEST_FILE") {
                if std::env::var_os("VARA_DATA_DIR").is_none() {
                    return Err("File harness requires isolated VARA_DATA_DIR".into());
                }
                let paths = if let Ok(batch) = std::env::var("VARA_TEST_FILES_JSON") {
                    serde_json::from_str::<Vec<String>>(&batch)?
                } else {
                    vec![path.to_string_lossy().to_string()]
                };
                let ids=if let Ok(id)=std::env::var("VARA_TEST_RETRY_ID"){
                    files.command("file_retry",serde_json::json!({"id":id}),&service.preferences()).map_err(std::io::Error::other)?;serde_json::json!([id])
                }else{files.command("file_add",serde_json::json!({"paths":paths,"engine":"local.parakeet","language":""}),&service.preferences()).map_err(std::io::Error::other)?};
                if std::env::var_os("VARA_TEST_CANCEL_FIRST").is_some() {
                    files
                        .command(
                            "file_cancel",
                            serde_json::json!({"id":ids[0]}),
                            &service.preferences(),
                        )
                        .map_err(std::io::Error::other)?;
                }
            }
            let _ = LIVE.set(Arc::downgrade(&service));
            app.manage(service.clone());
            let (sender, receiver) = std::sync::mpsc::channel();
            let _ = HOTKEY_EVENTS.set(sender);
            let weak = Arc::downgrade(&service);
            std::thread::spawn(move || run_shortcut_events(weak, receiver));
            let file_harness =
                cfg!(feature = "test-harness") && std::env::var_os("VARA_TEST_FILE").is_some();
            if let Err(error) = if file_harness {
                Ok(())
            } else {
                register_shortcut(app.handle(), &service.shortcut())
            } {
                service.warn(error);
            }
            let locale = vara_core::resolve_ui_locale(service.preferences().ui_locale, &vara_platform::preferred_languages());
            let native_menu = build_native_menu(app, locale)?;
            app.manage(native_menu);
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
        .build(app_context())
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
                app.state::<Arc<transcription::Files>>().stop();
                if let Some(service) = app.try_state::<Arc<Service>>() {
                    let _ = service.cancel();
                }
            }
            _ => {}
        });
}

/// Isolated native file adapter verification; excluded from normal builds.
#[cfg(all(feature = "test-harness", target_os = "macos"))]
pub fn run_native_file_fixture() -> bool {
    transcription::test_fixture_entry()
}

/// Isolated native verification entry; never included in the release feature set.
#[cfg(all(feature = "test-harness", target_os = "macos"))]
pub fn run_native_localization_fixture() {
    unsafe extern "C" {
        fn vara_fixture_probe_panel(
            message: *const std::ffi::c_char,
            extension: *const std::ffi::c_char,
            directory: *const std::ffi::c_char,
        );
        fn vara_fixture_panel_result() -> i32;
        fn vara_fixture_tray_tooltip(expected: *const std::ffi::c_char) -> i32;
    }
    let directory = std::env::temp_dir().join(format!("vara-native-dialog-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fixture directory");
    let directory_c = std::ffi::CString::new(directory.to_str().unwrap()).unwrap();
    let mut context = app_context();
    context.config_mut().app.windows.clear();
    tauri::Builder::default().setup(move |app| {
        let menu = build_native_menu(app, "en")?;
        assert_eq!(menu.show.text()?, "Open Vara");
        assert_eq!(menu.quit.text()?, "Quit Vara");
        assert_eq!(menu.show.id().as_ref(), "show");
        assert_eq!(menu.quit.id().as_ref(), "quit");
        menu.localize("da");
        assert_eq!(menu.show.text()?, "Åbn Vara");
        assert_eq!(menu.quit.text()?, "Afslut Vara");
        menu.localize("en");
        assert_eq!(menu.show.text()?, "Open Vara");
        assert_eq!(menu.quit.text()?, "Quit Vara");
        println!("PASS: actual Tauri tray menu handles initial and live locale changes");
        let handle = app.handle().clone();
        app.handle().run_on_main_thread(move || {
            std::thread::spawn(|| { std::thread::sleep(std::time::Duration::from_secs(30)); eprintln!("fixture watchdog timeout"); std::process::exit(2); });
            let en_tip = std::ffi::CString::new("Vara · dictation").unwrap();
            let da_tip = std::ffi::CString::new("Vara · diktering").unwrap();
            let en_seen = unsafe { vara_fixture_tray_tooltip(en_tip.as_ptr()) } == 1;
            menu.localize("da");
            let da_seen = unsafe { vara_fixture_tray_tooltip(da_tip.as_ptr()) } == 1;
            menu.localize("en");
            assert!(en_seen && da_seen, "status bar tooltip en={en_seen} da={da_seen}");
            println!("PASS: actual status bar tooltip en/da");
            for (locale, expected) in [("da", "Vælg lyd eller video"), ("en", "Choose audio or video")] {
                let expected = std::ffi::CString::new(expected).unwrap();
                unsafe { vara_fixture_probe_panel(expected.as_ptr(), std::ptr::null(), directory_c.as_ptr()); }
                assert!(transcription::pick_dialog(locale).set_directory(&directory).pick_files().is_none());
                assert_eq!(unsafe { vara_fixture_panel_result() }, 1, "pick panel {locale}");
            }
            for locale in ["da", "en"] {
                let ext = std::ffi::CString::new("txt").unwrap();
                unsafe { vara_fixture_probe_panel(std::ptr::null(), ext.as_ptr(), directory_c.as_ptr()); }
                assert!(transcription::export_dialog(locale, "txt", "fixture.txt").set_directory(&directory).save_file().is_none());
                assert_eq!(unsafe { vara_fixture_panel_result() }, 1, "save panel {locale}");
            }
            std::fs::remove_dir(&directory).expect("empty fixture directory");
            println!("PASS: actual RFD open/save panels da/en, localized message and preserved txt filter");
            handle.exit(0);
        })?;
        Ok(())
    }).run(context).expect("native fixture app");
}
