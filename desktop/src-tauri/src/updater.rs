use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tauri::{Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;
const ENDPOINT: &str =
    "https://github.com/mikkelkrogsholm/vara/releases/latest/download/latest.json";
#[derive(Deserialize, Serialize)]
#[serde(default)]
struct Preferences {
    automatic: bool,
    last_check: u64,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            automatic: true,
            last_check: 0,
        }
    }
}
struct Inner {
    preferences: Preferences,
    status: &'static str,
    pending: Option<tauri_plugin_updater::Update>,
    downloaded: Option<Vec<u8>>,
}
pub struct Updates {
    root: PathBuf,
    inner: Mutex<Inner>,
    operation: tokio::sync::Mutex<()>,
}
impl Updates {
    pub fn new(root: PathBuf) -> Self {
        let preferences = std::fs::read(root.join("updates.json"))
            .ok()
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();
        Self {
            root,
            operation: tokio::sync::Mutex::new(()),
            inner: Mutex::new(Inner {
                preferences,
                status: if key().is_some() { "idle" } else { "disabled" },
                pending: None,
                downloaded: None,
            }),
        }
    }
    fn snapshot(&self) -> Value {
        let i = self.inner.lock();
        json!({"status":i.status,"automatic":i.preferences.automatic,"version":i.pending.as_ref().map(|u| &u.version),"downloaded":i.downloaded.is_some()})
    }
    fn emit(&self, app: &tauri::AppHandle) {
        let _ = app.emit("vara:update", self.snapshot());
    }
    fn save(&self) -> Result<(), String> {
        let bytes = serde_json::to_vec(&self.inner.lock().preferences)
            .map_err(|_| crate::host_message("update.failed"))?;
        let temporary = self.root.join("updates.json.tmp");
        std::fs::write(&temporary, bytes)
            .and_then(|_| std::fs::rename(temporary, self.root.join("updates.json")))
            .map_err(|_| crate::host_message("update.failed"))
    }
}
fn key() -> Option<&'static str> {
    option_env!("VARA_UPDATER_PUBLIC_KEY")
        .map(str::trim)
        .filter(|k| !k.is_empty())
}
pub async fn command(
    app: &tauri::AppHandle,
    action: &str,
    payload: Value,
) -> Result<Value, String> {
    let updates = app.state::<Updates>();
    if action == "update_status" {
        return Ok(updates.snapshot());
    }
    let _operation = updates
        .operation
        .try_lock()
        .map_err(|_| crate::host_message("update.busy"))?;
    if action == "update_preference" {
        updates.inner.lock().preferences.automatic = payload["automatic"]
            .as_bool()
            .ok_or_else(|| crate::host_message("update.failed"))?;
        updates.save()?;
        return Ok(updates.snapshot());
    }
    let Some(key) = key() else {
        return Ok(updates.snapshot());
    };
    let result: Result<(), String> = async {
        match action {
            "update_check" => {
                let now = crate::service::now().max(0) as u64;
                {
                    let mut i = updates.inner.lock();
                    if !crate::activity::due(
                        i.preferences.automatic,
                        i.preferences.last_check,
                        now,
                        payload["manual"] == true,
                    ) {
                        return Ok(());
                    }
                    i.preferences.last_check = now;
                    i.status = "checking";
                }
                updates.save()?;
                updates.emit(app);
                let updater = app
                    .updater_builder()
                    .pubkey(key)
                    .endpoints(vec![ENDPOINT.parse().unwrap()])
                    .map_err(|_| crate::host_message("update.failed"))?
                    .timeout(Duration::from_secs(60))
                    .restart_after_install(false)
                    .build()
                    .map_err(|_| crate::host_message("update.failed"))?;
                let pending = updater
                    .check()
                    .await
                    .map_err(|_| crate::host_message("update.failed"))?;
                let mut i = updates.inner.lock();
                i.status = if pending.is_some() {
                    "available"
                } else {
                    "current"
                };
                i.pending = pending;
                i.downloaded = None;
            }
            "update_download" => {
                let service = app.state::<Arc<crate::service::Service>>();
                let files = app.state::<Arc<crate::transcription::Files>>();
                if service.update_busy() || files.update_busy()? {
                    return Err(crate::host_message("update.busy"));
                }
                let update = updates
                    .inner
                    .lock()
                    .pending
                    .clone()
                    .ok_or_else(|| crate::host_message("update.failed"))?;
                updates.inner.lock().status = "downloading";
                updates.emit(app);
                let bytes = update
                    .download(|_, _| {}, || {})
                    .await
                    .map_err(|_| crate::host_message("update.failed"))?;
                let mut i = updates.inner.lock();
                i.downloaded = Some(bytes);
                i.status = "ready";
            }
            "update_install" => {
                let service = app.state::<Arc<crate::service::Service>>();
                let files = app.state::<Arc<crate::transcription::Files>>();
                let update = updates
                    .inner
                    .lock()
                    .pending
                    .clone()
                    .ok_or_else(|| crate::host_message("update.failed"))?;
                // Close admission across GUI, native shortcut and the independent CLI owner.
                // No await between taking the gate, rechecking jobs and restarting.
                let _gate = crate::activity::install_guard(&updates.root, || {
                    Ok(service.update_busy() || files.update_busy()?)
                })
                .map_err(|_| crate::host_message("update.busy"))?;
                updates.inner.lock().status = "installing";
                updates.emit(app);
                let bytes = updates
                    .inner
                    .lock()
                    .downloaded
                    .take()
                    .ok_or_else(|| crate::host_message("update.failed"))?;
                update
                    .install(bytes)
                    .map_err(|_| crate::host_message("update.failed"))?;
                app.restart();
            }
            _ => return Err(crate::host_message("update.failed")),
        }
        Ok(())
    }
    .await;
    if let Err(error) = result {
        updates.inner.lock().status = if error == crate::host_message("update.busy") {
            "blocked"
        } else {
            "error"
        };
        updates.emit(app);
        return Err(error);
    }
    updates.emit(app);
    Ok(updates.snapshot())
}
