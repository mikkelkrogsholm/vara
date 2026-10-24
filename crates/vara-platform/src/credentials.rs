//! OS credentials never enter snapshots, settings or diagnostics.
use crate::Result;

fn validate(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 160 || id.contains('\0') {
        return Err("Ugyldigt forbindelses-id.".into());
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn entry(id: &str) -> Result<keyring::Entry> {
    validate(id)?;
    keyring::Entry::new("dk.vara.desktop", id)
        .map_err(|_| "Systemets nøglelager er ikke tilgængeligt. Åbn det og prøv igen.".into())
}

pub fn has_secret(id: &str) -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
        Ok(mac::request("status", id, "")?.exists)
    }
    #[cfg(not(target_os = "macos"))]
    {
        match entry(id)?.get_attributes() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(_) => {
                Err("Nøglelagerets status kunne ikke læses. Åbn nøglelageret og prøv igen.".into())
            }
        }
    }
}

pub fn secret(id: &str) -> Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        Ok(mac::request("get", id, "")?.value)
    }
    #[cfg(not(target_os = "macos"))]
    {
        match entry(id)?.get_password() {
            Ok(s) => Ok(Some(s)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => {
                Err("API-nøglen kunne ikke læses. Åbn systemets nøglelager og prøv igen.".into())
            }
        }
    }
}

pub fn set_secret(id: &str, key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() || key.len() > 4096 || key.chars().any(char::is_control) {
        return Err("Indtast en gyldig API-nøgle.".into());
    }
    #[cfg(target_os = "macos")]
    {
        mac::request("set", id, key.trim())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        entry(id)?.set_password(key.trim()).map_err(|_| {
            "API-nøglen kunne ikke gemmes. Åbn systemets nøglelager og prøv igen.".into()
        })
    }
}

pub fn delete_secret(id: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        mac::request("delete", id, "")?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        match entry(id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => {
                Err("API-nøglen kunne ikke slettes. Åbn systemets nøglelager og prøv igen.".into())
            }
        }
    }
}

/// Must run before Tauri initialization. The Mac worker accepts only a parent
/// satisfying its own designated signing requirement, never an arbitrary shell.
pub fn credential_worker_entry() -> bool {
    #[cfg(target_os = "macos")]
    {
        mac::worker_entry()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::{
        ffi::{CStr, CString},
        io::{Read, Write},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    const FLAG: &str = "--vara-credential-worker";
    const DEADLINE: Duration = Duration::from_secs(15);
    const UNAVAILABLE: &str =
        "Systemets nøglelager er ikke tilgængeligt. Åbn Nøglering og prøv igen.";
    unsafe extern "C" {
        fn vara_credential_parent_is_trusted() -> bool;
        fn vara_credential(
            operation: *const std::ffi::c_char,
            account: *const std::ffi::c_char,
            secret: *const std::ffi::c_char,
        ) -> *mut std::ffi::c_char;
        fn vara_free(value: *mut std::ffi::c_char);
    }
    #[derive(Serialize, Deserialize)]
    struct Request {
        operation: String,
        id: String,
        key: String,
    }
    #[derive(Serialize, Deserialize)]
    pub(super) struct Reply {
        status: i32,
        pub exists: bool,
        pub value: Option<String>,
    }

    pub(super) fn request(operation: &str, id: &str, key: &str) -> Result<Reply> {
        validate(id)?;
        let mut child = Command::new(std::env::current_exe().map_err(|_| UNAVAILABLE)?)
            .arg(FLAG)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| UNAVAILABLE)?;
        let input = serde_json::to_vec(&Request {
            operation: operation.into(),
            id: id.into(),
            key: key.into(),
        })
        .map_err(|_| UNAVAILABLE)?;
        let Some(mut stdin) = child.stdin.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(UNAVAILABLE.into());
        };
        // Pipe writes are also covered by the deadline. Killing/reaping the
        // worker closes its read end and releases a blocked writer.
        let writer = std::thread::spawn(move || stdin.write_all(&input));
        let result = finish(child, operation, DEADLINE);
        let written = matches!(writer.join(), Ok(Ok(())));
        match result {
            Err(error) => Err(error),
            Ok(reply) if written => Ok(reply),
            Ok(_) => Err(UNAVAILABLE.into()),
        }
    }
    fn finish(
        mut child: std::process::Child,
        operation: &str,
        deadline: Duration,
    ) -> Result<Reply> {
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(_)) => return Err(UNAVAILABLE.into()),
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(UNAVAILABLE.into());
                }
                Ok(None) => (),
            }
            if started.elapsed() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(if matches!(operation, "set" | "delete") {
                    "Nøglelageret svarede ikke inden 15 sekunder. Ændringen kan være gennemført. Åbn Nøglering, og brug Prøv igen for at kontrollere status, før du gentager ændringen."
                } else { "Nøglelageret svarede ikke inden 15 sekunder. Åbn Nøglering og prøv igen." }.into());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let reply: Reply =
            serde_json::from_reader(child.stdout.take().ok_or(UNAVAILABLE)?.take(32_768))
                .map_err(|_| UNAVAILABLE)?;
        match reply.status {
            0 | -25300 => Ok(reply), // errSecItemNotFound is an ordinary missing entry.
            -25308 | -25293 | -128 => Err("Nøgleringen er låst, eller Vara mangler adgang. Lås login-nøgleringen op i Nøglering, og kontrollér nøglens adgangskontrol for Vara. Prøv derefter igen.".into()),
            _ => Err(UNAVAILABLE.into()),
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn stalled_worker_is_killed_and_mutation_is_reported_as_uncertain() {
            for operation in ["status", "set", "delete"] {
                let child = Command::new("/bin/sleep")
                    .arg("10")
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();
                let pid = child.id();
                let started = Instant::now();
                let error = finish(child, operation, Duration::from_millis(20))
                    .err()
                    .unwrap();
                assert!(started.elapsed() < Duration::from_secs(2));
                assert_eq!(error.contains("kan være gennemført"), operation != "status");
                let still_exists = Command::new("/bin/kill")
                    .args(["-0", &pid.to_string()])
                    .stderr(Stdio::null())
                    .status()
                    .unwrap()
                    .success();
                assert!(!still_exists, "timed-out worker was not reaped");
            }
        }
    }
    pub(super) fn worker_entry() -> bool {
        if std::env::args().nth(1).as_deref() != Some(FLAG) {
            return false;
        }
        if !unsafe { vara_credential_parent_is_trusted() } {
            std::process::exit(77);
        }
        let request: Request = match serde_json::from_reader(std::io::stdin().take(32_768)) {
            Ok(value) => value,
            Err(_) => std::process::exit(64),
        };
        if validate(&request.id).is_err()
            || !matches!(
                request.operation.as_str(),
                "status" | "get" | "set" | "delete"
            )
        {
            std::process::exit(64);
        }
        let strings = [request.operation, request.id, request.key].map(CString::new);
        let [Ok(operation), Ok(id), Ok(key)] = strings else {
            std::process::exit(64);
        };
        unsafe {
            let result = vara_credential(operation.as_ptr(), id.as_ptr(), key.as_ptr());
            if result.is_null() {
                std::process::exit(70);
            }
            let written = std::io::stdout().write_all(CStr::from_ptr(result).to_bytes());
            vara_free(result);
            if written.is_err() {
                std::process::exit(74);
            }
        }
        true
    }
}
