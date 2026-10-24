//! Desktop capabilities. No platform assumptions escape these adapters into vara-core.
mod shortcut;
pub use shortcut::*;
mod recorder;
pub use recorder::*;
use serde::Serialize;
use std::ffi::{CStr, CString};
use vara_core::{Delivery, Destination};
pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Serialize)]
pub struct Permissions {
    pub microphone: String,
    pub accessibility: bool,
    pub automatic_insertion: bool,
    pub input_monitoring: Option<bool>,
}
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn vara_applications() -> *mut std::ffi::c_char;
    fn vara_capture_target() -> *mut std::ffi::c_char;
    fn vara_insert(token: *const std::ffi::c_char, text: *const std::ffi::c_char) -> i32;
    fn vara_release_target(token: *const std::ffi::c_char);
    fn vara_accessibility() -> i32;
    fn vara_input_monitoring() -> bool;
    fn vara_request_input_monitoring();
    fn vara_request_accessibility();
    fn vara_microphone() -> i32;
    fn vara_request_microphone();
    fn vara_hud(
        app_id: *const std::ffi::c_char,
        app_name: *const std::ffi::c_char,
        title: *const std::ffi::c_char,
        detail: *const std::ffi::c_char,
        recording: i32,
        cancel: extern "C" fn(),
        stop: extern "C" fn(),
        level: extern "C" fn() -> f32,
    );
    fn vara_hide_hud();
    fn vara_free(value: *mut std::ffi::c_char);
}
pub fn capture_target() -> Result<Destination> {
    #[cfg(target_os = "macos")]
    unsafe {
        let raw = vara_capture_target();
        if raw.is_null() {
            return Err("Den aktive app kunne ikke identificeres.".into());
        }
        let parsed = serde_json::from_slice(CStr::from_ptr(raw).to_bytes())
            .map_err(|_| "Den aktive app kunne ikke læses.".into());
        vara_free(raw);
        parsed
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(Destination {
            app_id: "unknown".into(),
            app_name: "Ukendt app".into(),
            target_token: None,
        })
    }
}
pub fn insert(origin: &Destination, text: &str) -> Delivery {
    #[cfg(target_os = "macos")]
    if let (Some(token), Ok(text)) = (&origin.target_token, CString::new(text)) {
        if let Ok(token) = CString::new(token.as_str()) {
            return match unsafe { vara_insert(token.as_ptr(), text.as_ptr()) } {
                1 => Delivery::Inserted,
                -1 => Delivery::Unknown,
                _ => Delivery::ManualReady,
            };
        }
    }
    Delivery::ManualReady
}
pub fn release_target(origin: &Destination) {
    #[cfg(target_os = "macos")]
    if let Some(token) = origin
        .target_token
        .as_ref()
        .and_then(|s| CString::new(s.as_str()).ok())
    {
        unsafe {
            vara_release_target(token.as_ptr());
        }
    }
}
pub fn permissions() -> Permissions {
    #[cfg(target_os = "macos")]
    unsafe {
        return Permissions {
            microphone: match vara_microphone() {
                1 => "authorized",
                0 => "not_determined",
                _ => "denied",
            }
            .into(),
            accessibility: vara_accessibility() == 1,
            automatic_insertion: true,
            input_monitoring: Some(vara_input_monitoring()),
        };
    }
    #[cfg(not(target_os = "macos"))]
    Permissions {
        microphone: "system".into(),
        accessibility: false,
        automatic_insertion: false,
        input_monitoring: None,
    }
}
pub fn request_permission(kind: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    unsafe {
        match kind {
            "microphone" => vara_request_microphone(),
            "accessibility" => vara_request_accessibility(),
            "input_monitoring" => vara_request_input_monitoring(),
            _ => return Err("Ukendt tilladelse.".into()),
        };
    }
    Ok(())
}
pub struct Hud<'a> {
    pub origin: &'a Destination,
    pub title: &'a str,
    pub detail: &'a str,
    pub recording: bool,
}
pub struct HudCallbacks {
    pub cancel: extern "C" fn(),
    pub stop: extern "C" fn(),
    pub level: extern "C" fn() -> f32,
}
pub fn hud(view: Hud<'_>, callbacks: HudCallbacks) {
    #[cfg(target_os = "macos")]
    if let (Ok(id), Ok(name), Ok(title), Ok(detail)) = (
        CString::new(view.origin.app_id.as_str()),
        CString::new(view.origin.app_name.as_str()),
        CString::new(view.title),
        CString::new(view.detail),
    ) {
        unsafe {
            vara_hud(
                id.as_ptr(),
                name.as_ptr(),
                title.as_ptr(),
                detail.as_ptr(),
                i32::from(view.recording),
                callbacks.cancel,
                callbacks.stop,
                callbacks.level,
            );
        }
    }
}
pub fn hide_hud() {
    #[cfg(target_os = "macos")]
    unsafe {
        vara_hide_hud();
    }
}
pub fn copy(text: &str) -> Result<()> {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(text))
        .map_err(|_| "Teksten kunne ikke kopieres. Markér den og brug systemets Kopiér.".into())
}
fn entry(id: &str) -> Result<keyring::Entry> {
    if id.is_empty() || id.len() > 160 {
        return Err("Ugyldigt forbindelses-id.".into());
    }
    keyring::Entry::new("dk.vara.desktop", id)
        .map_err(|_| "Systemets nøglelager er ikke tilgængeligt.".into())
}
pub fn secret(id: &str) -> Result<Option<String>> {
    match entry(id)?.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("API-nøglen kunne ikke læses i systemets nøglelager.".into()),
    }
}
pub fn set_secret(id: &str, key: &str) -> Result<()> {
    if key.trim().is_empty() {
        return Err("Indtast en API-nøgle.".into());
    }
    entry(id)?
        .set_password(key.trim())
        .map_err(|_| "API-nøglen kunne ikke gemmes i systemets nøglelager.".into())
}
pub fn delete_secret(id: &str) -> Result<()> {
    match entry(id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("API-nøglen kunne ikke slettes.".into()),
    }
}

pub fn applications() -> Result<serde_json::Value> {
    #[cfg(target_os = "macos")]
    unsafe {
        let raw = vara_applications();
        if raw.is_null() {
            return Err("Apps kunne ikke findes.".into());
        }
        let value = serde_json::from_slice(CStr::from_ptr(raw).to_bytes())
            .map_err(|_| "Apps kunne ikke læses.".into());
        vara_free(raw);
        value
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(serde_json::json!([]))
    }
}
