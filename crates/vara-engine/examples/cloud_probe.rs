//! Explicit developer live check. Credentials arrive via stdin, never CLI arguments or logs.
use serde_json::{Value, json};
use std::io::Read;
use vara_core::{Settings, TextPlan};
use vara_engine::{Cancellation, Engines};
#[tokio::main]
async fn main() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("Credential input unavailable");
        return;
    }
    let Ok(keys) = serde_json::from_str::<Value>(&input) else {
        eprintln!("Invalid credential input");
        return;
    };
    let scratch = tempfile::tempdir().unwrap();
    let engines = Engines::new(scratch.path().into()).unwrap();
    let settings = Settings::default();
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Samples/da_test.wav");
    let mut results = Vec::new();
    for provider in ["groq", "openai"] {
        let Some(key) = keys[provider].as_str().filter(|s| !s.is_empty()) else {
            results.push(json!({"provider":provider,"status":"no_accessible_key"}));
            continue;
        };
        let cancel = Cancellation::default();
        let models = if provider == "groq" {
            engines.groq_models(key, &cancel).await
        } else {
            engines.openai_models(key, &cancel).await
        };
        let mut result = json!({"provider":provider});
        match models {
            Ok(models) => {
                result["catalogue"] = json!("pass");
                result["configured_models"] = json!(settings.engines.iter().filter(|e| e.id.starts_with(&format!("{provider}."))).map(|e| json!({"id":e.id,"model":e.model,"available":models.contains(&e.model)})).collect::<Vec<_>>());
            }
            Err(error) => {
                result["catalogue_error"] = json!(error);
            }
        }
        for id in if provider == "groq" {
            vec!["groq.speech", "groq.speech.accurate"]
        } else {
            vec!["openai.speech"]
        } {
            let started = std::time::Instant::now();
            match engines
                .speech(
                    settings.engine(id).unwrap(),
                    &fixture,
                    Some("da"),
                    Some(key),
                    &cancel,
                )
                .await
            {
                Ok(text) => {
                    result[id] = json!({"status":"pass","characters":text.chars().count(),"seconds":started.elapsed().as_secs_f64()});
                }
                Err(error) => {
                    result[id] = json!({"status":"error","error":error});
                }
            }
        }
        let id = format!("{provider}.text");
        let plan = TextPlan {
            engine: settings.engine(&id).unwrap().clone(),
            system_prompt: "Ret kun tegnsætning. Returner kun teksten.".into(),
            app_specific: false,
        };
        let started = std::time::Instant::now();
        result[&id] = match engines
            .text(
                &plan,
                "hej jeg kommer i morgen klokken to",
                Some(key),
                &cancel,
            )
            .await
        {
            Ok(text) => {
                json!({"status":"pass","characters":text.chars().count(),"seconds":started.elapsed().as_secs_f64()})
            }
            Err(error) => json!({"status":"error","error":error}),
        };
        results.push(result);
    }
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}
