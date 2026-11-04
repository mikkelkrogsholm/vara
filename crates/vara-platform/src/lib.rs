//! Desktop capabilities. No platform assumptions escape these adapters into vara-core.
mod credentials;
pub use credentials::*;
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
    fn vara_preferred_languages() -> *mut std::ffi::c_char;
    fn vara_applications() -> *mut std::ffi::c_char;
    fn vara_application_icons(ids: *const std::ffi::c_char) -> *mut std::ffi::c_char;
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
/// Ordered OS language preferences; the core owns supported-language resolution.
pub fn preferred_languages() -> Vec<String> {
    #[cfg(target_os = "macos")]
    unsafe {
        let raw = vara_preferred_languages();
        if raw.is_null() {
            return Vec::new();
        }
        let languages = serde_json::from_slice(CStr::from_ptr(raw).to_bytes()).unwrap_or_default();
        vara_free(raw);
        languages
    }
    #[cfg(target_os = "windows")]
    unsafe {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetUserPreferredUILanguages(
                flags: u32,
                count: *mut u32,
                buffer: *mut u16,
                size: *mut u32,
            ) -> i32;
        }
        const MUI_LANGUAGE_NAME: u32 = 0x8;
        let (mut count, mut size) = (0, 0);
        if GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut count,
            std::ptr::null_mut(),
            &mut size,
        ) == 0
            || size == 0
        {
            return Vec::new();
        }
        let mut buffer = vec![0u16; size as usize];
        if GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut count,
            buffer.as_mut_ptr(),
            &mut size,
        ) == 0
        {
            return Vec::new();
        }
        buffer
            .split(|c| *c == 0)
            .filter(|v| !v.is_empty())
            .map(String::from_utf16_lossy)
            .collect()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let primary = std::env::var("LC_ALL")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("LC_MESSAGES").ok().filter(|s| !s.is_empty()))
            .or_else(|| std::env::var("LANG").ok())
            .unwrap_or_default();
        let languages = if primary == "C" || primary == "POSIX" || primary.starts_with("C.") {
            primary
        } else {
            std::env::var("LANGUAGE")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or(primary)
        };
        languages
            .split(':')
            .map(|s| s.split('.').next().unwrap_or("").to_string())
            .collect()
    }
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

/// Optional local artwork, addressed by app identity; absence uses a UI monogram.
pub fn application_icons(ids: &[String]) -> Result<serde_json::Value> {
    if ids.len() > 128
        || ids.iter().any(|id| {
            id.is_empty()
                || id.len() > 255
                || id.chars().any(|c| c.is_control() || c == '/' || c == '\\')
        })
    {
        return Err("Ugyldige app-id’er.".into());
    }
    #[cfg(target_os = "macos")]
    unsafe {
        let input =
            CString::new(serde_json::to_string(ids).map_err(|_| "App-id’er kunne ikke læses.")?)
                .map_err(|_| "Ugyldige app-id’er.")?;
        let raw = vara_application_icons(input.as_ptr());
        if raw.is_null() {
            return Err("Appikoner kunne ikke læses.".into());
        }
        let result = serde_json::from_slice(CStr::from_ptr(raw).to_bytes())
            .map_err(|_| "Appikoner kunne ikke læses.".into());
        vara_free(raw);
        result
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(serde_json::json!({}))
    }
}
