const KEY_CODES: &[(&str, u16)] = &[
    ("KeyA", 0),
    ("KeyS", 1),
    ("KeyD", 2),
    ("KeyF", 3),
    ("KeyH", 4),
    ("KeyG", 5),
    ("KeyZ", 6),
    ("KeyX", 7),
    ("KeyC", 8),
    ("KeyV", 9),
    ("IntlBackslash", 10),
    ("KeyB", 11),
    ("KeyQ", 12),
    ("KeyW", 13),
    ("KeyE", 14),
    ("KeyR", 15),
    ("KeyY", 16),
    ("KeyT", 17),
    ("Digit1", 18),
    ("Digit2", 19),
    ("Digit3", 20),
    ("Digit4", 21),
    ("Digit6", 22),
    ("Digit5", 23),
    ("Equal", 24),
    ("Digit9", 25),
    ("Digit7", 26),
    ("Minus", 27),
    ("Digit8", 28),
    ("Digit0", 29),
    ("BracketRight", 30),
    ("KeyO", 31),
    ("KeyU", 32),
    ("BracketLeft", 33),
    ("KeyI", 34),
    ("KeyP", 35),
    ("Enter", 36),
    ("KeyL", 37),
    ("KeyJ", 38),
    ("Quote", 39),
    ("KeyK", 40),
    ("Semicolon", 41),
    ("Backslash", 42),
    ("Comma", 43),
    ("Slash", 44),
    ("KeyN", 45),
    ("KeyM", 46),
    ("Period", 47),
    ("Tab", 48),
    ("Space", 49),
    ("Backquote", 50),
    ("Backspace", 51),
    ("Escape", 53),
    ("MetaRight", 54),
    ("MetaLeft", 55),
    ("ShiftLeft", 56),
    ("CapsLock", 57),
    ("AltLeft", 58),
    ("ControlLeft", 59),
    ("ShiftRight", 60),
    ("AltRight", 61),
    ("ControlRight", 62),
    ("Fn", 63),
    ("F17", 64),
    ("NumpadDecimal", 65),
    ("NumpadMultiply", 67),
    ("NumpadAdd", 69),
    ("NumLock", 71),
    ("AudioVolumeUp", 72),
    ("AudioVolumeDown", 73),
    ("AudioVolumeMute", 74),
    ("NumpadDivide", 75),
    ("NumpadEnter", 76),
    ("NumpadSubtract", 78),
    ("F18", 79),
    ("F19", 80),
    ("NumpadEqual", 81),
    ("Numpad0", 82),
    ("Numpad1", 83),
    ("Numpad2", 84),
    ("Numpad3", 85),
    ("Numpad4", 86),
    ("Numpad5", 87),
    ("Numpad6", 88),
    ("Numpad7", 89),
    ("F20", 90),
    ("Numpad8", 91),
    ("Numpad9", 92),
    ("IntlYen", 93),
    ("IntlRo", 94),
    ("F5", 96),
    ("F6", 97),
    ("F7", 98),
    ("F3", 99),
    ("F8", 100),
    ("F9", 101),
    ("F11", 103),
    ("F13", 105),
    ("F16", 106),
    ("F14", 107),
    ("F10", 109),
    ("F12", 111),
    ("F15", 113),
    ("Help", 114),
    ("Insert", 114),
    ("Home", 115),
    ("PageUp", 116),
    ("Delete", 117),
    ("F4", 118),
    ("End", 119),
    ("F2", 120),
    ("PageDown", 121),
    ("F1", 122),
    ("ArrowLeft", 123),
    ("ArrowRight", 124),
    ("ArrowDown", 125),
    ("ArrowUp", 126),
];
// Physical-key shortcut adapter; OS key codes stay outside the shared core.
use crate::Result;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn vara_shortcut_set(keys: *const u16, count: usize, callback: extern "C" fn(bool)) -> i32;
    fn vara_shortcut_clear();
    fn vara_shortcut_capture(handler: extern "C" fn(u16, bool)) -> i32;
    fn vara_shortcut_capture_end();
}

pub fn set_physical_shortcut(value: &str, callback: extern "C" fn(bool)) -> Result<()> {
    let keys = physical_keys(value)?;
    #[cfg(target_os = "macos")]
    {
        match unsafe { vara_shortcut_set(keys.as_ptr(), keys.len(), callback) } {
            1 => Ok(()),
            _ => Err("Genvejen kunne ikke aktiveres. Giv Vara adgang til Inputovervågning i Indstillinger → Tilladelser. Den tidligere genvej er bevaret.".into()),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (keys, callback);
        Err("Denne platformadapter understøtter endnu ikke fysiske flertastgenveje. Vælg en enkelt tast eller en kombination med Ctrl, Alt eller Shift.".into())
    }
}
pub fn clear_physical_shortcut() {
    #[cfg(target_os = "macos")]
    unsafe {
        vara_shortcut_clear()
    }
}
fn physical_keys(value: &str) -> Result<Vec<u16>> {
    let mut keys = Vec::new();
    for code in value
        .strip_prefix("Physical:")
        .ok_or("Ugyldig fysisk genvej.")?
        .split('+')
    {
        let key = keycode(code).ok_or_else(|| {
            format!("Tasten {code} kunne ikke knyttes til en fysisk tast på denne platform.")
        })?;
        if keys.contains(&key) {
            return Err("Genvejen indeholder samme tast flere gange.".into());
        }
        keys.push(key);
    }
    Ok(keys)
}
fn keycode(code: &str) -> Option<u16> {
    // macOS hardware key codes, independent of the active keyboard layout.
    KEY_CODES
        .iter()
        .find(|(name, _)| *name == code)
        .map(|(_, code)| *code)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn side_specific_modifiers_and_arbitrary_chords() {
        assert_eq!(
            physical_keys("Physical:AltLeft+MetaLeft").unwrap(),
            vec![58, 55]
        );
        assert_ne!(
            physical_keys("Physical:AltLeft").unwrap(),
            physical_keys("Physical:AltRight").unwrap()
        );
        assert_eq!(physical_keys("Physical:KeyA+KeyB").unwrap(), vec![0, 11]);
        assert_eq!(physical_keys("Physical:Fn").unwrap(), vec![63]);
        assert!(physical_keys("Physical:KeyA+KeyA").is_err());
        assert!(physical_keys("Physical:Unidentified").is_err());
    }
}

pub fn start_shortcut_capture(handler: extern "C" fn(u16, bool)) -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
        if unsafe { vara_shortcut_capture(handler) } == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = handler;
        Ok(false)
    }
}
pub fn end_shortcut_capture() {
    #[cfg(target_os = "macos")]
    unsafe {
        vara_shortcut_capture_end();
    }
}

pub fn physical_key_name(code: u16) -> Option<&'static str> {
    KEY_CODES
        .iter()
        .find(|(_, key)| *key == code)
        .map(|(name, _)| *name)
}
