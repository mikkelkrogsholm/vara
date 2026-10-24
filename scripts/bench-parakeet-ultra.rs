use parakeet_rs::{ExecutionConfig, ParakeetTDT, Transcriber};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let config = ExecutionConfig::new()
        .with_intra_threads(8)
        .with_environment(parakeet_rs::ort::init().build()?);
    let start = Instant::now();
    let mut model = ParakeetTDT::from_pretrained(&args[1], Some(config))?;
    let load_ms = start.elapsed().as_secs_f64() * 1000.;
    for run in 0..6 {
        let start = Instant::now();
        let result = model.transcribe_file(&args[2], None)?;
        println!(
            "{}",
            serde_json::json!({"device":"cpu","threads":8,"load_ms":load_ms,"run":run,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"text":result.text})
        );
    }
    Ok(())
}
