//! Explicit developer tooling, compiled only with `test-harness`.
//! Plans select baked-in public corpus IDs, never arbitrary upload paths or text.
use crate::{Cancellation, Engines};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use vara_core::{EngineConfig, Locality, Settings, TextPlan};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    models: PathBuf,
    rows: Vec<Row>,
    cloud_opt_in: bool,
    max_requests: usize,
    max_audio_seconds: f64,
    timeout_seconds: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: String,
    case: String,
    speech: Option<String>,
    text: Option<String>,
    profile: Option<String>,
    language: Option<String>,
}
fn error_code(error: &str) -> String {
    vara_core::UserMessage::decode(error)
        .map(|m| m.code)
        .unwrap_or("acceptance.internal".into())
}
fn cloud(engine: Option<&EngineConfig>) -> bool {
    engine.is_some_and(|e| e.locality == Locality::Cloud)
}

pub async fn run(
    path: &Path,
    secret: impl Fn(&EngineConfig) -> crate::Result<Option<String>>,
) -> Result<(), String> {
    let plan: Plan = serde_json::from_slice(&std::fs::read(path).map_err(|_| "plan read")?)
        .map_err(|_| "plan shape")?;
    if plan.rows.len() > 100
        || plan.max_requests > 40
        || !plan.max_audio_seconds.is_finite()
        || !(0.0..=180.0).contains(&plan.max_audio_seconds)
        || !(1..=120).contains(&plan.timeout_seconds)
    {
        return Err("plan ceilings".into());
    }
    let manifest: Value =
        serde_json::from_str(include_str!("../../../Samples/acceptance/manifest.json")).unwrap();
    let cleanup: Value =
        serde_json::from_str(include_str!("../../../Samples/acceptance/cleanup.json")).unwrap();
    let settings = Settings::default();
    let mut requests = 0;
    let mut seconds = 0.;
    // Validate the entire inventory before reading any credential or making any request.
    let mut row_ids = std::collections::BTreeSet::new();
    for row in &plan.rows {
        if row.id.is_empty() || !row_ids.insert(&row.id) {
            return Err("duplicate or empty row ID".into());
        }
        if row
            .language
            .as_deref()
            .is_some_and(|value| !["auto", "da", "en"].contains(&value))
        {
            return Err("language outside inventory".into());
        }
        if row.speech.is_none() && row.text.is_none() {
            return Err("empty row".into());
        }
        for id in row.speech.iter().chain(row.text.iter()) {
            if ![
                "local.parakeet",
                "local.whisper",
                "local.text",
                "openai.speech",
                "openai.text",
                "elevenlabs.speech",
                "gemini.speech",
                "gemini.text",
            ]
            .contains(&id.as_str())
            {
                return Err("engine outside inventory".into());
            }
        }
        let speech = row
            .speech
            .as_deref()
            .and_then(|id| settings.engine(id).ok());
        let text = row.text.as_deref().and_then(|id| settings.engine(id).ok());
        if speech.is_some_and(|e| e.kind != vara_core::EngineKind::Speech)
            || text.is_some_and(|e| e.kind != vara_core::EngineKind::Text)
        {
            return Err("engine kind".into());
        }
        if cloud(speech) || cloud(text) {
            if !plan.cloud_opt_in {
                return Err("cloud requires explicit opt in".into());
            }
            requests += usize::from(cloud(speech)) + usize::from(cloud(text));
        }
        if let Some(_) = speech {
            let case = manifest["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == row.case)
                .ok_or("unknown audio fixture")?;
            if cloud(speech) {
                let duration = case["duration_seconds"].as_f64().unwrap();
                if case["rights"]["cloud_upload_permitted"] != true || duration > 30. {
                    return Err("upload not authorized or too long".into());
                }
                seconds += duration;
            }
        } else if !cleanup["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == row.case)
        {
            return Err("unknown text fixture".into());
        }
        if text.is_some()
            && row
                .profile
                .as_deref()
                .and_then(|id| cleanup["profiles"][id].as_str())
                .is_none()
        {
            return Err("unknown prompt profile".into());
        }
    }
    if requests > plan.max_requests || seconds > plan.max_audio_seconds {
        return Err("inventory exceeds ceilings".into());
    }
    let mut engines = Engines::new(plan.models)?;
    engines.client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(plan.timeout_seconds))
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .map_err(|_| "client")?;
    let engines = Arc::new(engines);
    let mut used = std::collections::BTreeSet::new();
    for row in plan.rows {
        let start = Instant::now();
        let timestamp_unix_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "clock")?
            .as_secs();
        let speech = row
            .speech
            .as_deref()
            .and_then(|id| settings.engine(id).ok());
        let text_engine = row.text.as_deref().and_then(|id| settings.engine(id).ok());
        let phase = if row
            .speech
            .iter()
            .chain(row.text.iter())
            .any(|id| !used.contains(id))
        {
            "cold"
        } else {
            "warm"
        };
        let mut out = json!({"id":row.id,"case":row.case,"timestamp_unix_seconds":timestamp_unix_seconds,"speech":speech,"text_engine":text_engine,"profile":row.profile,"phase":phase,"raw":null,"final":null,"error":null,"transcript":null,"asr_seconds":null,"ai_seconds":null,"load_seconds":null,"load_note":"included in cold ASR/AI; not separately instrumented"});
        let cancel = Cancellation::default();
        let mut raw = String::new();
        let mut failed = false;
        if let Some(engine) = speech {
            let case = manifest["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == row.case)
                .unwrap();
            let audio = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../Samples/acceptance")
                .join(case["path"].as_str().unwrap());
            let bytes = std::fs::read(&audio).map_err(|_| "fixture read")?;
            if format!("{:x}", Sha256::digest(&bytes)) != case["sha256"].as_str().unwrap() {
                return Err("fixture hash mismatch".into());
            }
            let key = match secret(engine) {
                Ok(key) => key,
                Err(error) => {
                    out["error"] = json!(error_code(&error));
                    out["error_stage"] = json!("asr");
                    out["total_seconds"] = json!(start.elapsed().as_secs_f64());
                    println!("{}", out);
                    continue;
                }
            };
            let asr = Instant::now();
            let language = match row.language.as_deref() {
                Some("auto") => None,
                Some(value) => Some(value.to_owned()),
                None => case["language"]
                    .as_str()
                    .filter(|s| *s != "none" && *s != "da+en")
                    .map(str::to_owned),
            };
            out["language"] = json!(language);
            let result = if case.get("anchors").is_some() || case.get("turns").is_some() {
                let engines = engines.clone();
                let engine = engine.clone();
                let cancel = cancel.clone();
                tokio::task::spawn_blocking(move || {
                    engines.transcribe_file(
                        &engine,
                        &audio,
                        language.as_deref(),
                        key.as_deref(),
                        &cancel,
                        &|_| {},
                    )
                })
                .await
                .map_err(|_| "ASR job")?
                .map(|transcript| {
                    out["validation_error"] = json!(
                        transcript
                            .validate(case["duration_seconds"].as_f64().unwrap())
                            .err()
                            .map(|e| e.to_string())
                    );
                    raw = transcript.text.clone();
                    out["transcript"] = json!(transcript);
                    raw.clone()
                })
            } else if engine.provider == vara_core::Provider::Gemini {
                crate::remote::acceptance_gemini_speech(
                    &engines.client,
                    engine,
                    &audio,
                    language.as_deref(),
                    key.as_deref(),
                    &cancel,
                )
                .await
            } else {
                engines
                    .speech(engine, &audio, language.as_deref(), key.as_deref(), &cancel)
                    .await
            };
            out["asr_seconds"] = json!(asr.elapsed().as_secs_f64());
            match result {
                Ok(value) => {
                    raw = value;
                    out["raw"] = json!(raw);
                }
                Err(e) => {
                    failed = true;
                    out["error"] = json!(error_code(&e));
                    out["error_stage"] = json!("asr");
                }
            }
        } else {
            raw = cleanup["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == row.case)
                .unwrap()["input"]
                .as_str()
                .unwrap()
                .into();
            out["raw"] = json!(raw);
        }
        if !failed {
            if let Some(engine) = text_engine {
                // Existing adapters cap output: OpenAI 8192, Gemini 4096; inputs are bounded here.
                if raw.len() > 2048 {
                    return Err("text input ceiling".into());
                }
                let key = match secret(engine) {
                    Ok(key) => key,
                    Err(error) => {
                        out["error"] = json!(error_code(&error));
                        out["error_stage"] = json!("ai");
                        out["total_seconds"] = json!(start.elapsed().as_secs_f64());
                        println!("{}", out);
                        continue;
                    }
                };
                let text_plan = TextPlan {
                    engine: engine.clone(),
                    system_prompt: cleanup["profiles"][row.profile.as_deref().unwrap()]
                        .as_str()
                        .unwrap()
                        .into(),
                    app_specific: row.profile.as_deref() == Some("app_specific_prompt"),
                };
                let ai = Instant::now();
                match tokio::time::timeout(
                    std::time::Duration::from_secs(plan.timeout_seconds),
                    engines.text(&text_plan, &raw, key.as_deref(), &cancel),
                )
                .await
                {
                    Ok(Ok(value)) => out["final"] = json!(value),
                    Ok(Err(e)) => {
                        out["error"] = json!(error_code(&e));
                        out["error_stage"] = json!("ai");
                    }
                    Err(_) => {
                        cancel.cancel();
                        out["error"] = json!("acceptance.timeout");
                        out["error_stage"] = json!("ai");
                    }
                }
                out["ai_seconds"] = json!(ai.elapsed().as_secs_f64());
            } else {
                out["final"] = json!(raw);
            }
        }
        for id in row.speech.iter().chain(row.text.iter()) {
            used.insert(id.clone());
        }
        out["total_seconds"] = json!(start.elapsed().as_secs_f64());
        println!("{}", out);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn rejected(mut plan: Value, expected: &str) {
        plan["models"] = json!("/missing-models");
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), serde_json::to_vec(&plan).unwrap()).unwrap();
        let result = run(file.path(), |_| {
            panic!("credentials must not be read during rejection")
        })
        .await;
        assert_eq!(result.unwrap_err(), expected);
    }
    fn plan() -> Value {
        json!({"rows":[{"id":"one","case":"da-000","speech":"openai.speech","text":null,"profile":null}],"cloud_opt_in":false,"max_requests":1,"max_audio_seconds":13.0,"timeout_seconds":30})
    }
    #[tokio::test]
    async fn inaccessible_keys_do_not_abort_independent_rows() {
        let mut p = plan();
        p["models"] = json!("/unused-models");
        p["cloud_opt_in"] = json!(true);
        p["max_requests"] = json!(2);
        p["max_audio_seconds"] = json!(30);
        let row = p["rows"][0].clone();
        p["rows"].as_array_mut().unwrap().push(row);
        p["rows"][1]["id"] = json!("two");
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), serde_json::to_vec(&p).unwrap()).unwrap();
        let count = std::sync::atomic::AtomicUsize::new(0);
        run(file.path(), |_| {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(vara_core::UserMessage::new("credential.unavailable").wire())
        })
        .await
        .unwrap();
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
    #[tokio::test]
    async fn cloud_requires_opt_in_before_key_read() {
        rejected(plan(), "cloud requires explicit opt in").await;
    }
    #[tokio::test]
    async fn unknown_audio_is_rejected() {
        let mut p = plan();
        p["cloud_opt_in"] = json!(true);
        p["rows"][0]["case"] = json!("../private.wav");
        rejected(p, "unknown audio fixture").await;
    }
    #[tokio::test]
    async fn unknown_engine_is_rejected() {
        let mut p = plan();
        p["rows"][0]["speech"] = json!("custom.exfiltration");
        rejected(p, "engine outside inventory").await;
    }
    #[tokio::test]
    async fn request_ceiling_enforced() {
        let mut p = plan();
        p["cloud_opt_in"] = json!(true);
        p["max_requests"] = json!(0);
        rejected(p, "inventory exceeds ceilings").await;
    }
    #[tokio::test]
    async fn audio_ceiling_enforced() {
        let mut p = plan();
        p["cloud_opt_in"] = json!(true);
        p["max_audio_seconds"] = json!(1);
        rejected(p, "inventory exceeds ceilings").await;
    }
    #[tokio::test]
    async fn long_cloud_audio_refused() {
        let mut p = plan();
        p["cloud_opt_in"] = json!(true);
        p["rows"][0]["case"] = json!("long-boundaries");
        rejected(p, "upload not authorized or too long").await;
    }
    #[tokio::test]
    async fn engine_kind_is_checked() {
        let mut p = plan();
        p["rows"][0]["speech"] = json!("local.text");
        rejected(p, "engine kind").await;
    }
    #[tokio::test]
    async fn excessive_request_limit_refused() {
        let mut p = plan();
        p["max_requests"] = json!(41);
        rejected(p, "plan ceilings").await;
    }
    #[tokio::test]
    async fn excessive_timeout_refused() {
        let mut p = plan();
        p["timeout_seconds"] = json!(121);
        rejected(p, "plan ceilings").await;
    }
    #[tokio::test]
    async fn unknown_prompt_refused() {
        let mut p = plan();
        p["rows"][0]["speech"] = Value::Null;
        p["rows"][0]["text"] = json!("local.text");
        p["rows"][0]["case"] = json!("text-names-amount-date");
        p["rows"][0]["profile"] = json!("invented");
        rejected(p, "unknown prompt profile").await;
    }
}
