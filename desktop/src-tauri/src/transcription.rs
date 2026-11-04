use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager};
use vara_core::*;
type Result<T> = std::result::Result<T, String>;
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
    shared: Arc<vara_app::Files>,
}
impl Files {
    pub fn new(app: tauri::AppHandle, root: PathBuf) -> Result<Arc<Self>> {
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
            shared: vara_app::Files::new(host, root)?,
        }))
    }
    pub fn stop(&self) {
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
                let mut d = self
                    .shared
                    .document(p["id"].as_str().unwrap_or_default())
                    .ok_or(crate::host_message("file.document_missing"))?;
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
                            || same_file::is_same_file(&path, self.shared.path(&d.id))
                                .unwrap_or(false)
                        {
                            return Err(crate::host_message("file.export_overwrite").into());
                        }
                        let target = path.canonicalize().unwrap_or(path.clone());
                        if target
                            == PathBuf::from(&d.source_path)
                                .canonicalize()
                                .unwrap_or_default()
                            || target == self.shared.path(&d.id).canonicalize().unwrap_or_default()
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
            _ => return self.shared.command(action, p, settings),
        }
        Ok(Value::Null)
    }
}
