//! Developer verification only; not packaged as an end-user CLI.
use vara_core::*;
use vara_engine::{Cancellation, Engines};
#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = std::path::PathBuf::from(args.get(1).expect("model directory"));
    let audio = std::path::PathBuf::from(args.get(2).expect("fixture WAV"));
    let cancel = Cancellation::default();
    let settings = Settings::default();
    let ids = if args.iter().any(|a| a == "--text") {
        vec![settings.speech_engine.as_str(), "local.text"]
    } else {
        vec![settings.speech_engine.as_str()]
    };
    for id in ids {
        eprintln!("Installing/verifying {id}");
        vara_engine::install(id, &root, &cancel, |_| {})
            .await
            .expect("verified model");
    }
    let engines = Engines::new(root).unwrap();
    let start = std::time::Instant::now();
    let original = engines
        .speech(
            settings.engine(&settings.speech_engine).unwrap(),
            &audio,
            Some("da"),
            None,
            &cancel,
        )
        .await
        .expect("actual inference");
    println!("ASR ({:.2}s): {}", start.elapsed().as_secs_f64(), original);
    if args.iter().any(|a| a == "--text") {
        let plan = TextPlan {
            engine: settings.engine("local.text").unwrap().clone(),
            system_prompt: DEFAULT_PROMPT.into(),
            app_specific: false,
        };
        let start = std::time::Instant::now();
        let result = engines.text(&plan, &original, None, &cancel).await;
        println!("TEXT ({:.2}s): {:?}", start.elapsed().as_secs_f64(), result);
    }
}
