//! Verify the exact C ABI and subprocess used by the desktop host.
fn main() -> Result<(), String> {
    if vara_engine::who::worker_entry() {
        return Ok(());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        return Err("Usage: who RUNTIME MODEL_DIR status|download|diarize [NORMALIZED_WAV]".into());
    }
    if args[2] == "cancel" {
        let cancel = vara_engine::Cancellation::default();
        let other = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            other.cancel();
        });
        let result = vara_engine::who::run(
            std::path::Path::new(&args[0]),
            std::path::Path::new(&args[1]),
            "diarize",
            args.get(3).map(std::path::Path::new),
            &cancel,
        );
        assert_eq!(result.unwrap_err(), "Annulleret.");
        println!("{{\"cancelled\":true}}");
        return Ok(());
    }
    let known_count = args
        .get(4)
        .map(|s| {
            s.parse::<u32>()
                .map_err(|_| "Invalid speaker count".to_owned())
        })
        .transpose()?;
    let result = vara_engine::who::run_known(
        std::path::Path::new(&args[0]),
        std::path::Path::new(&args[1]),
        &args[2],
        args.get(3).map(std::path::Path::new),
        known_count,
        &Default::default(),
    )?;
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}
