//! Who's official SDK behind its pinned C ABI. Rust owns lifecycle, source safety
//! and cancellation; an isolated child contains native inference and SDK crashes.
use crate::{Cancellation, Result};
use std::{
    ffi::{CString, c_char, c_void},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use vara_core::{Diarization, SpeakerTurn};

pub const SDK_REVISION: &str = "86b9b55fe8c61ec1c27c5b6383f545c6df1d6ceb";
pub const MODEL_VERSION: &str = "v0.3.1";
pub fn library_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "libWhoAndroid.dylib"
    } else if cfg!(target_os = "windows") {
        "WhoAndroid.dll"
    } else {
        "libWhoAndroid.so"
    }
}
/// The caller supplies the packaged runtime path; no library search through PATH.
pub fn run(
    library: &Path,
    directory: &Path,
    operation: &str,
    audio: Option<&Path>,
    cancel: &Cancellation,
) -> Result<serde_json::Value> {
    if !library.is_file() {
        return Err(
            "Who-runtime mangler i denne installation. Installer den nyeste Vara-version.".into(),
        );
    }
    let temp = tempfile::tempdir().map_err(|_| "Who kunne ikke oprette en arbejdsmappe.")?;
    let output = temp.path().join("result.json");
    let mut command = Command::new(std::env::current_exe().map_err(|_| "Who kunne ikke starte.")?);
    command
        .arg("--vara-who-worker")
        .arg(library)
        .arg(directory)
        .arg(operation)
        .arg(audio.unwrap_or(Path::new("")))
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let child = cancel.spawn_child(|| command.spawn())?;
    let start = Instant::now();
    loop {
        if let Err(e) = cancel.check() {
            let mut p = child.lock();
            let _ = p.kill();
            let _ = p.wait();
            return Err(e);
        }
        if let Some(status) = child
            .lock()
            .try_wait()
            .map_err(|_| "Who-processen kunne ikke aflæses.")?
        {
            if !status.success() {
                return Err("Who kunne ikke køre. Runtime kræver macOS 15 eller nyere på Mac; lyd og tekst er bevaret.".into());
            }
            break;
        }
        if start.elapsed() > Duration::from_secs(1800) {
            let mut p = child.lock();
            let _ = p.kill();
            let _ = p.wait();
            return Err("Who tog for lang tid. Prøv igen; transskriptionen er bevaret.".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    cancel.check()?;
    let bytes = std::fs::read(output).map_err(|_| "Who returnerede intet resultat.")?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "Who returnerede et ugyldigt resultat.")?;
    if let Some(error) = value["error"].as_str() {
        return Err(error.to_owned());
    }
    Ok(value)
}
/// Call before starting any UI/runtime in executables that host Who jobs.
pub fn worker_entry() -> bool {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_none_or(|a| a != "--vara-who-worker") {
        return false;
    }
    if args.len() != 7 {
        std::process::exit(2);
    }
    let result = unsafe {
        native(
            Path::new(&args[2]),
            Path::new(&args[3]),
            &args[4].to_string_lossy(),
            Path::new(&args[5]),
        )
    };
    let value = match result {
        Ok(v) => v,
        Err(e) => serde_json::json!({"error":e}),
    };
    if std::fs::write(&args[6], serde_json::to_vec(&value).unwrap()).is_err() {
        std::process::exit(3);
    }
    true
}
// The SDK owns all returned allocation; its matching free/destroy functions are
// always called before the library drops. Only its pinned schema is accepted.
unsafe fn native(
    library: &Path,
    directory: &Path,
    operation: &str,
    audio: &Path,
) -> Result<serde_json::Value> {
    type Create = unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> *mut c_void;
    type State = unsafe extern "C" fn(*mut c_void) -> i32;
    type Run = unsafe extern "C" fn(
        *mut c_void,
        *const u8,
        i32,
        *const u8,
        i32,
        *const c_char,
        *const c_char,
    ) -> *mut u8;
    type Destroy = unsafe extern "C" fn(*mut c_void);
    type Free = unsafe extern "C" fn(*mut u8);
    type Flush = unsafe extern "C" fn();
    // SAFETY: this path is an explicit signed/bundled SDK artifact, not discovered
    // from model files or the working directory. The ABI is pinned in the build.
    let lib = unsafe { libloading::Library::new(library) }
        .map_err(|_| "Who-runtime kunne ikke indlæses.")?;
    let create: libloading::Symbol<Create> =
        unsafe { lib.get(b"who_create") }.map_err(|_| "Who-runtime er inkompatibel.")?;
    let state: libloading::Symbol<State> =
        unsafe { lib.get(b"dal_is_downloaded") }.map_err(|_| "Who-runtime er inkompatibel.")?;
    let download: libloading::Symbol<State> =
        unsafe { lib.get(b"dal_download") }.map_err(|_| "Who-runtime er inkompatibel.")?;
    let run: libloading::Symbol<Run> =
        unsafe { lib.get(b"dal_run") }.map_err(|_| "Who-runtime er inkompatibel.")?;
    let destroy: libloading::Symbol<Destroy> =
        unsafe { lib.get(b"dal_destroy") }.map_err(|_| "Who-runtime er inkompatibel.")?;
    let free: libloading::Symbol<Free> =
        unsafe { lib.get(b"dal_buffer_free") }.map_err(|_| "Who-runtime er inkompatibel.")?;
    let flush: libloading::Symbol<Flush> =
        unsafe { lib.get(b"dal_flush_telemetry") }.map_err(|_| "Who-runtime er inkompatibel.")?;
    let name = CString::new("who").unwrap();
    let dir =
        CString::new(directory.to_string_lossy().as_bytes()).map_err(|_| "Ugyldig modelsti.")?;
    let handle = unsafe { create(name.as_ptr(), std::ptr::null(), dir.as_ptr()) };
    if handle.is_null() {
        return Err("Who kunne ikke oprettes.".into());
    }
    let result = (|| -> Result<serde_json::Value> {
        if operation == "status" {
            return Ok(serde_json::json!({"available":true,"ready":unsafe{state(handle)}==1}));
        }
        if operation == "download" {
            if unsafe { download(handle) } != 0 {
                return Err("Who kunne ikke hentes eller klargøres. Kontroller netværk og ledig diskplads, og prøv igen.".into());
            }
            return Ok(serde_json::json!({"ready":unsafe{state(handle)}==1}));
        }
        if operation != "diarize" {
            return Err("Ukendt Who-operation.".into());
        }
        if unsafe { state(handle) } != 1 {
            return Err("Hent Who-modellen først.".into());
        }
        let mut wav = hound::WavReader::open(audio).map_err(|_| "Afspilningskopien mangler.")?;
        let spec = wav.spec();
        if spec.channels != 1
            || spec.sample_rate != 16000
            || spec.bits_per_sample != 16
            || wav.duration() > 16000 * 28800
        {
            return Err("Who kræver Varas normaliserede lydkopi (højst 8 timer).".into());
        }
        let count = wav.duration();
        let mut input = Vec::with_capacity(count as usize * 4 + 12);
        input.extend_from_slice(&count.to_be_bytes());
        for sample in wav.samples::<i16>() {
            let v = sample.map_err(|_| "Lydkopien kunne ikke læses.")? as f32 / 32768.;
            input.extend_from_slice(&v.to_bits().to_be_bytes());
        }
        input.extend_from_slice(&16000_f64.to_bits().to_be_bytes());
        let ptr = unsafe {
            run(
                handle,
                input.as_ptr(),
                i32::try_from(input.len()).map_err(|_| "Filen er for stor til Who.")?,
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        if ptr.is_null() {
            return Err("Who kunne ikke finde talere. Transskriptionen er bevaret; prøv igen med en anden fil.".into());
        }
        let len = u32::from_be_bytes(
            unsafe { std::slice::from_raw_parts(ptr, 4) }
                .try_into()
                .unwrap(),
        ) as usize;
        let result = if (16..=32_000_000).contains(&len) {
            decode(
                unsafe { std::slice::from_raw_parts(ptr.add(4), len) },
                count as f64 / 16000.,
            )
        } else {
            Err("Who returnerede ugyldige talerdata.".into())
        };
        unsafe { free(ptr) };
        Ok(serde_json::to_value(result?).unwrap())
    })();
    // Preserve the SDK's licensing/usage reporting, including short-lived workers.
    unsafe {
        flush();
        destroy(handle);
    }
    result
}
fn decode(bytes: &[u8], duration: f64) -> Result<Diarization> {
    let bad = || "Who returnerede ugyldige talerdata.".to_owned();
    if bytes.len() < 16 {
        return Err(bad());
    }
    let speakers = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let count = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if count.checked_mul(20).and_then(|n| n.checked_add(16)) != Some(bytes.len()) {
        return Err(bad());
    }
    let mut turns = Vec::with_capacity(count);
    for chunk in bytes[8..8 + count * 20].chunks_exact(20) {
        let speaker = u32::from_be_bytes(chunk[..4].try_into().unwrap());
        if speaker >= speakers {
            return Err(bad());
        }
        turns.push(SpeakerTurn {
            speaker: format!("who-{speaker}"),
            start: f64::from_be_bytes(chunk[4..12].try_into().unwrap()),
            end: f64::from_be_bytes(chunk[12..20].try_into().unwrap()),
        });
    }
    let reported = f64::from_be_bytes(bytes[bytes.len() - 8..].try_into().unwrap());
    if !reported.is_finite() || (reported - duration).abs() > 0.1 {
        return Err(bad());
    }
    let result = Diarization {
        model: "Who by Desert Ant Labs".into(),
        version: MODEL_VERSION.into(),
        turns,
    };
    result.validate(duration).map_err(|e| e.to_string())?;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_malformed_ffi_before_using_turns() {
        assert!(decode(&[], 1.).is_err());
        let mut b = vec![0; 16];
        b[3] = 1;
        b[7] = 1;
        assert!(decode(&b, 1.).is_err());
        let mut b = vec![];
        b.extend(1_u32.to_be_bytes());
        b.extend(1_u32.to_be_bytes());
        b.extend(0_u32.to_be_bytes());
        b.extend(0_f64.to_be_bytes());
        b.extend(1_f64.to_be_bytes());
        b.extend(1_f64.to_be_bytes());
        assert_eq!(decode(&b, 1.).unwrap().turns.len(), 1);
        b[20..28].copy_from_slice(&f64::NAN.to_be_bytes());
        assert!(decode(&b, 1.).is_err());
    }
}
