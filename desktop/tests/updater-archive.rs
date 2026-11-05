// Diagnostic reproduction of tauri-plugin-updater 2.14.1 macOS extraction.
// This does not install, rename or launch an app. Destination must not exist.
use std::{env, fs, io, path::PathBuf};
use flate2::read::GzDecoder;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 3 { return Err("usage: updater-archive ARCHIVE NEW_TEMP_DIRECTORY".into()); }
    let destination = PathBuf::from(&args[2]);
    if !destination.starts_with(env::temp_dir()) { return Err("destination must be below the system temporary directory".into()); }
    fs::create_dir(&destination)?;
    let bytes = fs::read(&args[1])?;
    let mut archive = tar::Archive::new(GzDecoder::new(io::Cursor::new(bytes)));
    let mut count = 0;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let original = entry.path()?.to_path_buf();
        let collected: PathBuf = original.iter().skip(1).collect();
        let target = destination.join(&collected);
        if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
        if let Err(error) = entry.unpack(&target) {
            eprintln!("FAIL entry={original:?} type={:?} mapped={collected:?} kind={:?} raw={:?}: {error}", entry.header().entry_type(), error.kind(), error.raw_os_error());
            return Err(error.into());
        }
        count += 1;
    }
    println!("PASS: {count} entries unpacked with the plugin's exact skip(1)/unpack algorithm into {}", destination.display());
    Ok(())
}
