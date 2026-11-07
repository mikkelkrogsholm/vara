use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager};
use vara_core::*;
type Result<T> = std::result::Result<T, String>;
pub(crate) fn allow_media(app: &tauri::AppHandle, root: &std::path::Path) -> tauri::Result<()> {
    app.asset_protocol_scope()
        .allow_directory(root.join("transcriptions"), false)
}
pub(crate) fn pick_dialog(locale: &str) -> rfd::FileDialog {
    rfd::FileDialog::new().set_title(UserMessage::new("native.file.pick").format(locale))
}
pub(crate) fn export_dialog(locale: &str, format: &str, name: &str) -> rfd::FileDialog {
    rfd::FileDialog::new().set_file_name(name).add_filter(
        &UserMessage::new("native.file.transcription").format(locale),
        &[format],
    )
}
pub struct Files {
    root_gate: PathBuf,
    #[cfg(not(target_os = "macos"))]
    shared: Arc<vara_app::Files>,
    #[cfg(target_os = "macos")]
    root: PathBuf,
    #[cfg(target_os = "macos")]
    executable: PathBuf,
    #[cfg(target_os = "macos")]
    running: Arc<std::sync::atomic::AtomicBool>,
}
impl Files {
    pub fn new(app: tauri::AppHandle, root: PathBuf) -> Result<Arc<Self>> {
        #[cfg(target_os = "macos")]
        {
            let executable =
                std::env::current_exe().map_err(|_| backend_message("backend.start_failed"))?;
            let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
            let files = Arc::new(Self {
                root_gate: root.clone(),
                root,
                executable,
                running: running.clone(),
            });
            let weak = Arc::downgrade(&files);
            std::thread::spawn(move || {
                let mut observed: Option<(String, u64)> = None;
                let mut noticed: Option<(String, u64)> = None;
                while running.load(std::sync::atomic::Ordering::SeqCst) {
                    let Some(files) = weak.upgrade() else { break };
                    if let Ok(response) = crate::local_backend::call(
                        &files.root,
                        crate::local_backend::Operation::Ping,
                        &files.executable,
                    ) {
                        let version = (response.instance.clone(), response.revision);
                        if observed.as_ref() != Some(&version) {
                            observed = Some(version);
                            let _ = app.emit("vara:files", ());
                        }
                        if let Some(notice) = response.notice {
                            let key = (response.instance, notice.revision);
                            if noticed.as_ref() != Some(&key) {
                                noticed = Some(key);
                                let _ = app.emit("vara:file-error", notice.message.wire());
                            }
                        }
                    }
                    drop(files);
                    std::thread::sleep(std::time::Duration::from_millis(750));
                }
            });
            return Ok(files);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let resource_dir = app.path().resource_dir().ok();
            let host = vara_app::Host {
                resource_dir,
                secret: Box::new(crate::service::engine_secret),
                notify: Box::new(move |error| {
                    if let Some(error) = error {
                        let _ = app.emit("vara:file-error", error);
                    } else {
                        let _ = app.emit("vara:files", ());
                    }
                }),
            };
            Ok(Arc::new(Self {
                root_gate: root.clone(),
                shared: vara_app::Files::new(host, root)?,
            }))
        }
    }
    pub fn update_busy(&self) -> Result<bool> {
        #[cfg(target_os = "macos")]
        {
            let response = crate::local_backend::call(&self.root, crate::local_backend::Operation::Ping, &self.executable)?;
            response.result.and_then(|v| v["busy"].as_bool())
                .ok_or_else(|| crate::host_message("update.busy"))
        }
        #[cfg(not(target_os = "macos"))]
        { Ok(self.shared.busy()) }
    }
    pub fn stop(&self) {
        #[cfg(target_os = "macos")]
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        #[cfg(not(target_os = "macos"))]
        self.shared.stop();
    }
    pub fn command(self: &Arc<Self>, action: &str, p: Value, settings: &Settings) -> Result<Value> {
        match action {
            "file_who_link" => {
                let url = match p["target"].as_str() {
                    Some("license") => "https://license.desertant.com/1.0",
                    Some("vendor") => "https://desertant.com",
                    _ => return Err(crate::host_message("file.who_link_unknown").into()),
                };
                #[cfg(target_os = "macos")]
                let result = std::process::Command::new("/usr/bin/open").arg(url).spawn();
                #[cfg(target_os = "windows")]
                let result = std::process::Command::new("cmd")
                    .args(["/C", "start", "", url])
                    .spawn();
                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                let result = std::process::Command::new("xdg-open").arg(url).spawn();
                result.map_err(|e| crate::host_io("file.link_open", &e))?;
            }
            "file_pick" => {
                let locale =
                    resolve_ui_locale(settings.ui_locale, &vara_platform::preferred_languages());
                let files = pick_dialog(locale).pick_files().unwrap_or_default();
                let list=files.into_iter().filter(|p|p.is_file()).map(|p|json!({"path":p.to_string_lossy(),"name":p.file_name().unwrap_or_default().to_string_lossy(),"bytes":p.metadata().map(|m|m.len()).unwrap_or(0)})).collect::<Vec<_>>();
                return Ok(json!(list));
            }
            "file_copy" | "file_export" => {
                let mut d = self.document(p["id"].as_str().unwrap_or_default())?;
                let format = p["format"].as_str().unwrap_or("txt");
                let original = p["original"] == true;
                let unsaved_draft = !original && p["draft"].is_string();
                if unsaved_draft {
                    let text = p["draft"].as_str().unwrap();
                    if text.len() > 4_000_000 {
                        return Err(crate::host_message("file.draft_limit").into());
                    }
                    d.edited_text = Some(text.into());
                }
                let text = if p["ai"] == true {
                    let text = d
                        .ai_text
                        .clone()
                        .ok_or(crate::host_message("file.no_ai_text"))?;
                    match format {
                        "txt" => text,
                        "md" => format!("# {} — AI\n\n{}\n", d.title, text),
                        "json" => d.export("json", true).map_err(|e| e.wire())?,
                        _ => return Err(crate::host_message("file.ai_export_format").into()),
                    }
                } else {
                    d.export(format, original).map_err(|e| e.wire())?
                };
                let text = if format == "json" && unsaved_draft {
                    let mut json: Value = serde_json::from_str(&text)
                        .map_err(|e| crate::host_json("file.export_serialize", &e))?;
                    json["unsaved_draft"] = json!(true);
                    json.to_string()
                } else {
                    text
                };
                if action == "file_copy" {
                    vara_platform::copy(&text)?;
                } else {
                    let safe = d
                        .title
                        .chars()
                        .map(|c| {
                            if c.is_alphanumeric() || " -_".contains(c) {
                                c
                            } else {
                                '_'
                            }
                        })
                        .collect::<String>();
                    let locale = resolve_ui_locale(
                        settings.ui_locale,
                        &vara_platform::preferred_languages(),
                    );
                    let path =
                        export_dialog(locale, format, &format!("{safe}.{format}")).save_file();
                    if let Some(path) = path {
                        if same_file::is_same_file(&path, &d.source_path).unwrap_or(false)
                            || same_file::is_same_file(&path, self.path(&d.id)).unwrap_or(false)
                        {
                            return Err(crate::host_message("file.export_overwrite").into());
                        }
                        let target = path.canonicalize().unwrap_or(path.clone());
                        if target
                            == PathBuf::from(&d.source_path)
                                .canonicalize()
                                .unwrap_or_default()
                            || target == self.path(&d.id).canonicalize().unwrap_or_default()
                        {
                            return Err(crate::host_message("file.export_source").into());
                        }
                        std::fs::write(path, text)
                            .map_err(|e| crate::host_io("file.export_save", &e))?;
                    } else {
                        return Ok(json!(false));
                    }
                }
            }
            _ => return self.remote_or_shared(action, p, settings),
        }
        Ok(Value::Null)
    }
    #[cfg(target_os = "macos")]
    fn remote_or_shared(&self, action: &str, mut p: Value, _settings: &Settings) -> Result<Value> {
        let name = action
            .strip_prefix("file_")
            .ok_or_else(|| backend_message("backend.operation_failed"))?;
        if !matches!(
            name,
            "list"
                | "add"
                | "cancel"
                | "retry"
                | "delete"
                | "persist"
                | "save"
                | "rename"
                | "speaker"
                | "ai"
                | "ai_cancel"
                | "who_status"
                | "who"
                | "who_download"
                | "who_cancel"
        ) {
            return Err(backend_message("backend.operation_failed"));
        }
        if p.is_null() {
            p = json!({});
        }
        let object = p
            .as_object_mut()
            .ok_or_else(|| backend_message("backend.malformed"))?;
        object.insert("action".into(), json!(name));
        let op = serde_json::from_value(p).map_err(|_| backend_message("backend.malformed"))?;
        let response = crate::local_backend::call(&self.root, op, &self.executable)
            .map_err(|e| backend_message(&e))?;
        if let Some(error) = response.error {
            return Err(error
                .message
                .map(|m| m.wire())
                .unwrap_or_else(|| backend_message(&error.code)));
        }
        // JSON null is the successful result of commands without a payload.
        Ok(response.result.unwrap_or(Value::Null))
    }
    #[cfg(not(target_os = "macos"))]
    fn remote_or_shared(&self, action: &str, p: Value, settings: &Settings) -> Result<Value> {
        let _activity = crate::activity::shared(&self.root_gate)
            .map_err(|_| crate::host_message("update.busy"))?;
        self.shared.command(action, p, settings)
    }
    #[cfg(target_os = "macos")]
    fn document(&self, id: &str) -> Result<TranscriptionDocument> {
        let list = self.remote_or_shared("file_list", json!({}), &Settings::default())?;
        let value = list["documents"]
            .as_array()
            .and_then(|v| v.iter().find(|d| d["id"] == id))
            .cloned()
            .ok_or(crate::host_message("file.document_missing"))?;
        serde_json::from_value(value).map_err(|_| backend_message("backend.response_invalid"))
    }
    #[cfg(not(target_os = "macos"))]
    fn document(&self, id: &str) -> Result<TranscriptionDocument> {
        self.shared
            .document(id)
            .ok_or(crate::host_message("file.document_missing"))
    }
    #[cfg(target_os = "macos")]
    fn path(&self, id: &str) -> PathBuf {
        self.root.join("transcriptions").join(format!("{id}.wav"))
    }
    #[cfg(not(target_os = "macos"))]
    fn path(&self, id: &str) -> PathBuf {
        self.shared.path(id)
    }
}

#[cfg(target_os = "macos")]
fn backend_message(code: &str) -> String {
    let key = match code {
        "backend.permissions" | "backend.peer_denied" => "backend.access_failed",
        "backend.frame_limit" => "backend.request_too_large",
        "backend.version_mismatch" => "backend.incompatible",
        "backend.operation_failed" => "backend.operation_failed",
        _ => "backend.connection_failed",
    };
    UserMessage::new(key).wire()
}

// Native integration fixture: a real Tauri AppHandle and the production adapter,
// with no Service, shortcuts, microphone or personal profile.
#[cfg(all(feature = "test-harness", target_os = "macos"))]
pub fn test_fixture_entry() -> bool {
    use crate::local_backend::{Operation, call};
    use tauri::Listener;
    let mut args = std::env::args_os();
    let _ = args.next();
    let mode = args.next();
    if mode.as_deref() == Some(std::ffi::OsStr::new("--vara-test-file-snapshot")) {
        let root = PathBuf::from(args.next().expect("isolated profile"));
        let mut context = crate::app_context();
        context.config_mut().app.windows.clear();
        tauri::Builder::default().setup(move |app| {
            let handle = app.handle().clone();
            allow_media(&handle, &root)?;
            let files = Files::new(handle.clone(), root.clone()).map_err(std::io::Error::other)?;
            std::thread::spawn(move || {
                let outcome = files.command("file_list", Value::Null, &Settings::default());
                files.stop();
                match outcome {
                    Ok(value) => { println!("{value}"); handle.exit(0); }
                    Err(error) => { eprintln!("{error}"); handle.exit(1); }
                }
            });
            Ok(())
        }).build(context).expect("snapshot fixture app").run(|_, _| {});
        return true;
    }
    if mode.as_deref() == Some(std::ffi::OsStr::new("--vara-test-adapter-client")) {
        let root = PathBuf::from(args.next().expect("profile"));
        let source = PathBuf::from(args.next().expect("source"));
        let response = call(
            &root,
            Operation::Submit {
                request: vara_app::SubmitRequest {
                    source,
                    engine_id: Some("openai.speech".into()),
                    language: None,
                    request_id: Some("adapter-fixture".into()),
                    diarize: false,
                    speakers: None,
                },
            },
            &std::env::current_exe().unwrap(),
        )
        .unwrap();
        assert!(response.error.is_none(), "{:?}", response.error);
        println!("{}", response.result.unwrap());
        return true;
    }
    if mode.as_deref() == Some(std::ffi::OsStr::new("--vara-test-adapter-status")) {
        let root = PathBuf::from(args.next().expect("profile"));
        let id = args.next().expect("id").to_string_lossy().to_string();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            let response = call(
                &root,
                Operation::Status { id: id.clone() },
                &std::env::current_exe().unwrap(),
            )
            .unwrap();
            let doc = response.result.unwrap();
            if doc["stage"] == "failed" {
                assert_eq!(
                    UserMessage::decode(doc["error"].as_str().unwrap())
                        .unwrap()
                        .code,
                    "provider.missing_key"
                );
                println!("accepted GUI job continued after exit: {}", doc["stage"]);
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "job did not complete: {doc}"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        return true;
    }
    if mode.as_deref() != Some(std::ffi::OsStr::new("--vara-test-adapter")) {
        return false;
    }
    let root = PathBuf::from(args.next().expect("isolated profile"));
    let source = PathBuf::from(args.next().expect("synthetic wav"));
    let mut context = crate::app_context();
    context.config_mut().app.windows.clear();
    tauri::Builder::default().setup(move |app| {
        let handle = app.handle().clone();
        allow_media(&handle, &root)?;
        let (tx, rx) = std::sync::mpsc::channel();
        let changes = tx.clone();
        app.listen("vara:files", move |_| { let _ = changes.send("files"); });
        app.listen("vara:file-error", move |_| { let _ = tx.send("error"); });
        let files = Files::new(handle.clone(), root.clone()).map_err(std::io::Error::other)?;
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let settings = Settings::default();
                let initial = files.command("file_list", Value::Null, &settings).unwrap();
                assert!(initial["documents"].as_array().unwrap().is_empty());
                let first = call(&root, Operation::Ping, &std::env::current_exe().unwrap()).unwrap();
                let child = std::process::Command::new(std::env::current_exe().unwrap())
                    .arg("--vara-test-adapter-client").arg(&root).arg(&source).output().unwrap();
                assert!(child.status.success(), "{}", String::from_utf8_lossy(&child.stderr));
                let receipt: Value = serde_json::from_slice(&child.stdout).unwrap();
                let id = receipt["job_id"].as_str().expect("job ID");
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
                loop {
                    let list = files.command("file_list", Value::Null, &settings).unwrap();
                    if list["documents"].as_array().unwrap().iter().any(|d| d["id"] == id) { break; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                let media = loop {
                    let list = files.command("file_list", Value::Null, &settings).unwrap();
                    if let Some(media) = list["media"][id].as_str() { break media.to_owned(); }
                    assert!(std::time::Instant::now() < deadline, "backend media path");
                    std::thread::sleep(std::time::Duration::from_millis(50));
                };
                assert_eq!(std::fs::canonicalize(&media).unwrap().parent().unwrap(), std::fs::canonicalize(root.join("transcriptions")).unwrap());
                assert!(handle.asset_protocol_scope().is_allowed(&media));
                assert!(!handle.asset_protocol_scope().is_allowed(&source));
                let mut file_events = 0;
                let mut notices = 0;
                let end = std::time::Instant::now() + std::time::Duration::from_secs(3);
                while std::time::Instant::now() < end {
                    match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                        Ok("files") => file_events += 1,
                        Ok("error") => notices += 1,
                        _ => {}
                    }
                }
                assert!(file_events >= 1 && file_events < 8, "file events: {file_events}");
                assert_eq!(notices, 1, "notice deduplication");
                // Wait for the first synthetic job to finish, then let the worker idle out.
                loop {
                    let status = call(&root, Operation::Status { id: id.into() }, &std::env::current_exe().unwrap()).unwrap();
                    if status.result.unwrap()["stage"] == "failed" { break; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                files.stop();
                std::thread::sleep(std::time::Duration::from_secs(4));
                // Seed a completed transcript without any inference so Save exercises the
                // actual adapter and 24 MB JSON-escaped request across the socket.
                let store = vara_core::TranscriptionStore::open(&root.join("transcriptions.sqlite3")).unwrap();
                let mut doc = store.all().unwrap().into_iter().find(|d| d.id == id).unwrap();
                doc.stage = FileStage::Ready;
                doc.error = None;
                doc.transcript = Some(Transcript { text: "base".into(), timing: Timing::None, timing_source: "fixture".into(), segments: vec![] });
                store.save(&doc).unwrap();
                drop(store);
                let restarted = call(&root, Operation::Ping, &std::env::current_exe().unwrap()).unwrap();
                assert_ne!(first.instance, restarted.instance, "backend must restart after idle");
                while rx.try_recv().is_ok() {}
                let second = Files::new(handle.clone(), root.clone()).unwrap();
                let list = second.command("file_list", Value::Null, &settings).unwrap();
                assert!(list["documents"].as_array().unwrap().iter().any(|d| d["id"] == id));
                let mut restart_files = 0;
                let mut restart_notices = 0;
                let end = std::time::Instant::now() + std::time::Duration::from_secs(2);
                while std::time::Instant::now() < end {
                    match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                        Ok("files") => restart_files += 1,
                        Ok("error") => restart_notices += 1,
                        _ => {}
                    }
                }
                assert_eq!(restart_files, 1, "one reload on backend restart");
                assert_eq!(restart_notices, 1, "one notice for the new instance");
                let edited = "\0".repeat(4_000_000);
                let saved = second.command("file_save", json!({"id": id, "revision": doc.revision, "text": edited}), &settings).unwrap();
                assert_eq!(saved["revision"], doc.revision + 1);
                let status = call(&root, Operation::Status { id: id.into() }, &std::env::current_exe().unwrap()).unwrap().result.unwrap();
                assert_eq!(status["edited_text"].as_str().unwrap().len(), 4_000_000);
                // A synchronous AI request survives beyond the framing timeout. A newly
                // connected GUI sees it as busy and can cancel through a second socket.
                let ai_adapter = second.clone();
                let ai_id = id.to_owned();
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                let ai_thread = std::thread::spawn(move || {
                    let result = ai_adapter.command("file_ai", json!({"id":ai_id,"engine":"openai.text","prompt":"fixture"}), &Settings::default());
                    done_tx.send(result).unwrap();
                });
                let wait = std::time::Instant::now() + std::time::Duration::from_secs(5);
                loop {
                    let list = second.command("file_list", Value::Null, &settings).unwrap();
                    if list["documents"].as_array().unwrap().iter().any(|d| d["id"] == id && d["ai_busy"] == true) { break; }
                    assert!(std::time::Instant::now() < wait, "AI never became busy");
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                assert!(matches!(done_rx.recv_timeout(std::time::Duration::from_secs(31)), Err(std::sync::mpsc::RecvTimeoutError::Timeout)), "AI returned before cancellation");
                let reopened = Files::new(handle.clone(), root.clone()).unwrap();
                let list = reopened.command("file_list", Value::Null, &settings).unwrap();
                assert!(list["documents"].as_array().unwrap().iter().any(|d| d["id"] == id && d["ai_busy"] == true));
                reopened.command("file_ai_cancel", json!({"id": id}), &settings).unwrap();
                let error = done_rx.recv_timeout(std::time::Duration::from_secs(15)).unwrap().unwrap_err();
                assert_eq!(UserMessage::decode(&error).unwrap().code, "runtime.cancelled");
                ai_thread.join().unwrap();
                let list = reopened.command("file_list", Value::Null, &settings).unwrap();
                assert!(list["documents"].as_array().unwrap().iter().any(|d| d["id"] == id && d["ai_busy"] == false));
                reopened.stop();
                let ids = second.command("file_add", json!({"paths":[source], "engine":"openai.speech", "language":""}), &settings).unwrap();
                let gui_id = ids[0].as_str().unwrap();
                let status = call(&root, Operation::Status { id: gui_id.into() }, &std::env::current_exe().unwrap()).unwrap().result.unwrap();
                assert!(status["stage"] == "queued" || status["stage"] == "transcribing", "{status}");
                std::fs::write(root.join("gui-job-id"), gui_id).unwrap();
                second.stop();
                println!("adapter fixture passed: external submit, shared document, events, notice, idle restart, 4 MB escaped save, long AI reconnect/cancel, GUI submit");
            }));
            if outcome.is_err() { handle.exit(1); } else { handle.exit(0); }
        });
        Ok(())
    }).build(context).expect("fixture app").run(|_, _| {});
    true
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    fn backend_errors_are_localized_messages() {
        for code in [
            "backend.profile_unavailable",
            "backend.permissions",
            "backend.peer_denied",
            "backend.unavailable",
            "backend.transport",
            "backend.timeout",
            "backend.malformed",
            "backend.frame_limit",
            "backend.already_running",
            "backend.start_failed",
            "backend.response_invalid",
            "backend.version_mismatch",
            "backend.operation_failed",
        ] {
            let message = UserMessage::decode(&backend_message(code)).unwrap();
            for locale in ["da", "en"] {
                let text = message.format(locale);
                assert!(
                    !text.is_empty() && !text.contains("backend."),
                    "{code} {locale}: {text}"
                );
            }
        }
    }
}
