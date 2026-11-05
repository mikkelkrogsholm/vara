//! Cross-process admission gate: jobs hold a shared lock while being accepted;
//! installation holds the exclusive lock, then checks already accepted work.
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
fn open(root: &Path) -> std::io::Result<File> {
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("update.lock"))
}
pub fn shared(root: &Path) -> Result<File, String> {
    let file = open(root).map_err(|e| e.to_string())?;
    file.try_lock_shared().map_err(|e| e.to_string())?;
    Ok(file)
}
pub fn exclusive(root: &Path) -> Result<File, String> {
    let file = open(root).map_err(|e| e.to_string())?;
    file.try_lock().map_err(|e| e.to_string())?;
    Ok(file)
}
pub fn install_guard(
    root: &Path,
    busy: impl FnOnce() -> Result<bool, String>,
) -> Result<File, String> {
    let gate = exclusive(root)?;
    if busy()? {
        return Err("busy".into());
    }
    Ok(gate)
}
pub fn due(automatic: bool, last: u64, now: u64, manual: bool) -> bool {
    manual || (automatic && (last == 0 || now.saturating_sub(last) >= 86_400))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn daily_and_manual() {
        assert!(due(true, 0, 1, false));
        assert!(!due(false, 0, 1, false));
        assert!(!due(true, 100, 101, false));
        assert!(due(true, 100, 86500, false));
        assert!(!due(true, 100, 99, false));
        assert!(due(false, 100, 101, true));
    }
    #[test]
    fn accepted_work_and_unknown_evidence_block_install() {
        let root = std::env::temp_dir().join(format!("vara-busy-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for (dictation, files, models) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            assert!(install_guard(&root, || Ok(dictation || files || models)).is_err());
            assert!(shared(&root).is_ok(), "failure releases admission gate");
        }
        assert!(install_guard(&root, || Err("unknown backend".into())).is_err());
        let gate = install_guard(&root, || Ok(false)).unwrap();
        assert!(shared(&root).is_err());
        drop(gate);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn admission_and_install_are_exclusive() {
        let root = std::env::temp_dir().join(format!("vara-gate-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let job = shared(&root).unwrap();
        assert!(exclusive(&root).is_err());
        drop(job);
        let install = exclusive(&root).unwrap();
        assert!(shared(&root).is_err());
        assert!(exclusive(&root).is_err());
        drop(install);
        assert!(shared(&root).is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }
}
