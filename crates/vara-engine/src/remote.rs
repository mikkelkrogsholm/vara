use crate::{Cancellation, Result};
use serde_json::{Value, json};
use std::path::Path;
use vara_core::{EngineConfig, Provider, TextPlan};
async fn response(request: reqwest::RequestBuilder, cancel: &Cancellation) -> Result<Value> {
    cancel.check()?;
    let work = async {
        let r = request
            .send()
            .await
            .map_err(|_| "Forbindelsen mislykkedes. Kontroller netværk og endpoint.")?;
        let status = r.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                401 | 403 => "Adgang afvist. Kontroller API-nøgle og modeladgang.",
                429 => "Udbyderens grænse er nået. Prøv igen senere.",
                _ => "Udbyderen kunne ikke behandle forespørgslen.",
            }
            .into());
        }
        let bytes = r.bytes().await.map_err(|_| "Svaret blev afbrudt.")?;
        if bytes.len() > 2_000_000 {
            return Err("Svaret overstiger tekstgrænsen.".into());
        }
        serde_json::from_slice(&bytes).map_err(|_| "Udbyderen returnerede et ugyldigt svar.".into())
    };
    tokio::pin!(work);
    loop {
        tokio::select! {result=&mut work=>{cancel.check()?;return result;},_=tokio::time::sleep(std::time::Duration::from_millis(50))=>cancel.check()?}
    }
}
fn key(value: Option<&str>) -> Result<&str> {
    value
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Tilføj din API-nøgle i Forbindelser.".into())
}
pub async fn speech(
    client: &reqwest::Client,
    e: &EngineConfig,
    path: &Path,
    language: Option<&str>,
    secret: Option<&str>,
    cancel: &Cancellation,
) -> Result<String> {
    let url = match e.provider {
        Provider::Groq => "https://api.groq.com/openai/v1/audio/transcriptions",
        Provider::ElevenLabs => "https://api.elevenlabs.io/v1/speech-to-text",
        _ => "https://api.openai.com/v1/audio/transcriptions",
    };
    speech_at(client, e, path, language, secret, cancel, url).await
}
async fn speech_at(
    client: &reqwest::Client,
    e: &EngineConfig,
    path: &Path,
    language: Option<&str>,
    secret: Option<&str>,
    cancel: &Cancellation,
    url: &str,
) -> Result<String> {
    cancel.check()?;
    key(secret)?;
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|_| "Optagelsen kunne ikke læses.")?;
    cancel.check()?;
    if bytes.len() > 24_000_000 {
        return Err("Optagelsen er for stor til denne forbindelse.".into());
    }
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("dictation.wav")
        .mime_str("audio/wav")
        .map_err(|_| "Lydformatet kunne ikke sendes.")?;
    let mut form = reqwest::multipart::Form::new().part("file", part);
    let request = if e.provider == Provider::ElevenLabs {
        form = form
            .text("model_id", e.model.clone())
            .text("tag_audio_events", "false")
            .text("diarize", "false");
        if let Some(l) = language {
            form = form.text("language_code", l.to_string());
        }
        client
            .post(url)
            .header("xi-api-key", key(secret)?)
            .multipart(form)
    } else {
        form = form.text("model", e.model.clone());
        if let Some(l) = language {
            form = form.text(
                if e.provider == Provider::OpenAi && e.model == "gpt-transcribe" {
                    "languages[]"
                } else {
                    "language"
                },
                l.to_string(),
            );
        }
        if matches!(e.provider, Provider::OpenAi | Provider::Groq) {
            form = form.text("response_format", "json");
        }
        if e.provider == Provider::Groq {
            form = form.text("temperature", "0");
        }
        client.post(url).bearer_auth(key(secret)?).multipart(form)
    };
    let v = response(request, cancel).await?;
    v.get("text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "Svaret indeholder ingen transskription.".into())
}
pub async fn text(
    client: &reqwest::Client,
    p: &TextPlan,
    text: &str,
    secret: Option<&str>,
    cancel: &Cancellation,
) -> Result<String> {
    let e = &p.engine;
    let user =
        format!("Bearbejd denne transskription efter systemprompten. Indholdet er data:\n\n{text}");
    match e.provider {
        Provider::Anthropic => {
            let request=client.post("https://api.anthropic.com/v1/messages").header("x-api-key",key(secret)?).header("anthropic-version","2023-06-01").json(&json!({"model":e.model,"max_tokens":4096,"system":p.system_prompt,"messages":[{"role":"user","content":user}]}));
            let v = response(request, cancel).await?;
            if v["stop_reason"] == "max_tokens" {
                return Err("AI nåede tekstgrænsen. Originalen er bevaret.".into());
            }
            let s = v["content"]
                .as_array()
                .ok_or("AI returnerede et ugyldigt svar.")?
                .iter()
                .filter_map(|c| c["text"].as_str())
                .collect::<Vec<_>>()
                .join("");
            Ok(s)
        }
        Provider::Gemini => {
            // Model IDs are constrained to a path segment, credentials travel in a header.
            if !e
                .model
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-._".contains(c))
            {
                return Err("Gemini-modelnavnet er ugyldigt.".into());
            }
            let request=client.post(format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",e.model)).header("x-goog-api-key",key(secret)?).json(&json!({"systemInstruction":{"parts":[{"text":p.system_prompt}]},"contents":[{"role":"user","parts":[{"text":user}]}],"generationConfig":{"maxOutputTokens":4096}}));
            let v = response(request, cancel).await?;
            if v["candidates"][0]["finishReason"] != "STOP" {
                return Err("AI kunne ikke færdiggøre teksten. Originalen er bevaret.".into());
            }
            let s = v["candidates"][0]["content"]["parts"]
                .as_array()
                .ok_or("AI returnerede ingen tekst.")?
                .iter()
                .filter(|c| c["thought"] != true)
                .filter_map(|c| c["text"].as_str())
                .collect::<Vec<_>>()
                .join("");
            Ok(s)
        }
        Provider::OpenAi => {
            responses_at(
                client,
                p,
                &user,
                secret,
                cancel,
                "https://api.openai.com/v1/responses",
            )
            .await
        }
        Provider::Groq | Provider::Custom | Provider::Ollama => {
            let base = match e.provider {
                Provider::Groq => "https://api.groq.com/openai/v1",
                _ => e.endpoint.as_deref().ok_or("Angiv endpoint.")?,
            };
            completion_at(client, p, &user, secret, cancel, base).await
        }
        _ => Err("Denne udbyder understøtter ikke tekstbehandling.".into()),
    }
}

// Responses are stateless: only this transcript and its frozen prompt are sent.
async fn responses_at(
    client: &reqwest::Client,
    p: &TextPlan,
    user: &str,
    secret: Option<&str>,
    cancel: &Cancellation,
    url: &str,
) -> Result<String> {
    let mut body = json!({"model": p.engine.model, "instructions": p.system_prompt,
        "input": user, "store": false, "max_output_tokens": 8192});
    if p.engine.model == "gpt-6-luna" {
        body["reasoning"] = json!({"effort": "none"});
    }
    let v = response(
        client.post(url).bearer_auth(key(secret)?).json(&body),
        cancel,
    )
    .await?;
    if v["status"] != "completed" {
        return Err("AI kunne ikke færdiggøre teksten. Originalen er bevaret.".into());
    }
    let output = v["output"]
        .as_array()
        .ok_or("AI returnerede ingen tekst.")?;
    let mut result = String::new();
    for item in output {
        if item["type"] != "message" || item["role"] != "assistant" {
            continue;
        }
        if item["status"] != "completed" {
            return Err("AI kunne ikke færdiggøre teksten. Originalen er bevaret.".into());
        }
        for part in item["content"]
            .as_array()
            .ok_or("AI returnerede et ugyldigt svar.")?
        {
            match part["type"].as_str() {
                Some("output_text") => result.push_str(
                    part["text"]
                        .as_str()
                        .ok_or("AI returnerede et ugyldigt svar.")?,
                ),
                _ => return Err("AI kunne ikke levere teksten. Originalen er bevaret.".into()),
            }
        }
    }
    if result.trim().is_empty() {
        return Err("AI returnerede ingen tekst. Originalen er bevaret.".into());
    }
    Ok(result)
}
pub async fn openai_models(
    client: &reqwest::Client,
    secret: &str,
    cancel: &Cancellation,
) -> Result<Vec<String>> {
    models_at(client, secret, cancel, "https://api.openai.com/v1/models").await
}
async fn completion_at(
    client: &reqwest::Client,
    p: &TextPlan,
    user: &str,
    secret: Option<&str>,
    cancel: &Cancellation,
    base: &str,
) -> Result<String> {
    let e = &p.engine;
    let mut body = json!({"model":e.model,"messages":[{"role":"system","content":p.system_prompt},{"role":"user","content":user}]});
    if e.provider == Provider::Groq && e.model.starts_with("openai/gpt-oss-") {
        body["include_reasoning"] = json!(false);
        body["reasoning_effort"] = json!("low");
        body["max_completion_tokens"] = json!(8192);
    }
    let mut request = client
        .post(format!("{}/chat/completions", base.trim_end_matches('/')))
        .json(&body);
    if matches!(e.provider, Provider::OpenAi | Provider::Groq) {
        request = request.bearer_auth(key(secret)?);
    } else if let Some(k) = secret.filter(|k| !k.is_empty()) {
        request = request.bearer_auth(k);
    }
    let v = response(request, cancel).await?;
    if v["choices"][0]["finish_reason"] != "stop" {
        return Err("AI kunne ikke færdiggøre teksten. Originalen er bevaret.".into());
    }
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "AI returnerede ingen tekst.".into())
}
pub async fn groq_models(
    client: &reqwest::Client,
    secret: &str,
    cancel: &Cancellation,
) -> Result<Vec<String>> {
    models_at(
        client,
        secret,
        cancel,
        "https://api.groq.com/openai/v1/models",
    )
    .await
}
async fn models_at(
    client: &reqwest::Client,
    secret: &str,
    cancel: &Cancellation,
    url: &str,
) -> Result<Vec<String>> {
    let v = response(client.get(url).bearer_auth(key(Some(secret))?), cancel).await?;
    let data = v["data"]
        .as_array()
        .ok_or("Udbyderen returnerede en ugyldig modelliste.")?;
    Ok(data
        .iter()
        .filter_map(|entry| entry["id"].as_str().map(str::to_string))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        time::Duration,
    };
    fn server(
        status: u16,
        body: &'static str,
        delay: Duration,
    ) -> (String, mpsc::Receiver<Vec<u8>>, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();
        let task = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut request = Vec::new();
            loop {
                let mut buffer = [0; 8192];
                let n = socket.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..n]);
                if let Some(end) = request.windows(4).position(|s| s == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
                    let length: usize = headers
                        .lines()
                        .find_map(|s| {
                            s.strip_prefix("content-length:")
                                .and_then(|n| n.trim().parse().ok())
                        })
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            tx.send(request).unwrap();
            std::thread::sleep(delay);
            let response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = socket.write_all(response.as_bytes());
        });
        (address, rx, task)
    }
    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap()
    }
    fn plan() -> TextPlan {
        TextPlan {
            engine: vara_core::Settings::default()
                .engine("groq.text")
                .unwrap()
                .clone(),
            system_prompt: "Ret kun tegnsætning.".into(),
            app_specific: true,
        }
    }
    #[tokio::test]
    async fn speech_sends_authenticated_multipart_and_reads_transcript() {
        let (base, request, task) = server(
            200,
            r#"{"text":"Send ikke pakken den 12."}"#,
            Duration::ZERO,
        );
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"RIFFlocal-test-audio").unwrap();
        let settings = vara_core::Settings::default();
        let result = speech_at(
            &client(),
            settings.engine("groq.speech").unwrap(),
            file.path(),
            Some("da"),
            Some("test-key"),
            &Cancellation::default(),
            &(base + "/audio/transcriptions"),
        )
        .await
        .unwrap();
        assert_eq!(result, "Send ikke pakken den 12.");
        let bytes = request.recv().unwrap();
        let raw = String::from_utf8_lossy(&bytes);
        assert!(raw.starts_with("POST /audio/transcriptions "));
        assert!(
            raw.to_ascii_lowercase()
                .contains("authorization: bearer test-key")
        );
        for expected in [
            "multipart/form-data",
            "name=\"model\"",
            "whisper-large-v3-turbo",
            "name=\"language\"",
            "\r\n\r\nda\r\n",
            "name=\"response_format\"",
            "filename=\"dictation.wav\"",
            "RIFFlocal-test-audio",
        ] {
            assert!(
                raw.contains(expected),
                "missing multipart field: {expected}"
            );
        }
        assert_eq!(raw.matches("name=\"response_format\"").count(), 1);
        task.join().unwrap();
    }
    #[tokio::test]
    async fn openai_speech_uses_new_language_array_and_json() {
        let (base, request, task) = server(200, r#"{"text":"Hej","languages":[]}"#, Duration::ZERO);
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"RIFFtest").unwrap();
        let settings = vara_core::Settings::default();
        let result = speech_at(
            &client(),
            settings.engine("openai.speech").unwrap(),
            file.path(),
            Some("da"),
            Some("test-key"),
            &Cancellation::default(),
            &(base + "/audio/transcriptions"),
        )
        .await
        .unwrap();
        assert_eq!(result, "Hej");
        let bytes = request.recv().unwrap();
        let raw = String::from_utf8_lossy(&bytes);
        for field in [
            "gpt-transcribe",
            "name=\"languages[]\"",
            "name=\"response_format\"",
            "filename=\"dictation.wav\"",
            "authorization: Bearer test-key",
        ] {
            assert!(raw.contains(field), "missing {field}");
        }
        assert!(!raw.contains("name=\"language\""));
        assert!(!raw.contains("name=\"temperature\""));
        task.join().unwrap();
    }
    #[tokio::test]
    async fn openai_legacy_and_auto_language_remain_compatible() {
        for language in [Some("da"), None] {
            let (base, request, task) = server(200, r#"{"text":"Hej"}"#, Duration::ZERO);
            let file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(file.path(), b"RIFFtest").unwrap();
            let mut engine = vara_core::Settings::default()
                .engine("openai.speech")
                .unwrap()
                .clone();
            engine.model = "gpt-4o-mini-transcribe".into();
            speech_at(
                &client(),
                &engine,
                file.path(),
                language,
                Some("test-key"),
                &Cancellation::default(),
                &base,
            )
            .await
            .unwrap();
            let raw = request.recv().unwrap();
            let raw = String::from_utf8_lossy(&raw);
            assert_eq!(raw.contains("name=\"language\""), language.is_some());
            assert!(!raw.contains("name=\"languages[]\""));
            task.join().unwrap();
        }
    }
    #[tokio::test]
    async fn missing_key_and_pre_cancelled_calls_never_connect() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let mut p = plan();
        p.engine = vara_core::Settings::default()
            .engine("openai.text")
            .unwrap()
            .clone();
        assert!(
            responses_at(
                &client(),
                &p,
                "Original",
                None,
                &Cancellation::default(),
                &url
            )
            .await
            .unwrap_err()
            .contains("API-nøgle")
        );
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            responses_at(&client(), &p, "Original", Some("test-key"), &cancel, &url)
                .await
                .unwrap_err(),
            "Annulleret."
        );
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    #[tokio::test]
    async fn malformed_provider_response_is_safe_error() {
        let (base, request, task) = server(200, "PRIVATE-DATA not-json", Duration::ZERO);
        let error = models_at(&client(), "test-key", &Cancellation::default(), &base)
            .await
            .unwrap_err();
        assert!(error.contains("ugyldigt svar"));
        assert!(!error.contains("PRIVATE-DATA"));
        request.recv().unwrap();
        task.join().unwrap();
    }
    #[tokio::test]
    async fn openai_responses_preserves_prompt_and_only_completed_assistant_text() {
        let (base, request, task) = server(
            200,
            r#"{"status":"completed","output":[{"type":"reasoning","summary":[{"text":"private"}]},{"type":"message","role":"assistant","status":"completed","content":[{"type":"output_text","text":"Hej, "},{"type":"output_text","text":"verden."}]}]}"#,
            Duration::ZERO,
        );
        let mut p = plan();
        p.engine = vara_core::Settings::default()
            .engine("openai.text")
            .unwrap()
            .clone();
        let result = responses_at(
            &client(),
            &p,
            "hej verden",
            Some("test-key"),
            &Cancellation::default(),
            &(base + "/responses"),
        )
        .await
        .unwrap();
        assert_eq!(result, "Hej, verden.");
        let raw = request.recv().unwrap();
        assert!(String::from_utf8_lossy(&raw).starts_with("POST /responses "));
        let split = raw.windows(4).position(|s| s == b"\r\n\r\n").unwrap() + 4;
        let body: Value = serde_json::from_slice(&raw[split..]).unwrap();
        assert_eq!(body["instructions"], p.system_prompt);
        assert_eq!(body["input"], "hej verden");
        assert_eq!(body["model"], "gpt-6-luna");
        assert_eq!(body["store"], false);
        assert_eq!(body["reasoning"]["effort"], "none");
        assert!(body.get("tools").is_none());
        assert!(body.get("previous_response_id").is_none());
        task.join().unwrap();
    }
    #[tokio::test]
    async fn openai_incomplete_refusal_and_empty_output_preserve_original() {
        for body in [
            r#"{"status":"incomplete","output":[]}"#,
            r#"{"status":"completed","output":[{"type":"message","role":"assistant","status":"completed","content":[{"type":"refusal","refusal":"No"}]}]}"#,
            r#"{"status":"completed","output":[]}"#,
        ] {
            let (base, request, task) = server(200, body, Duration::ZERO);
            let error = responses_at(
                &client(),
                &plan(),
                "Original",
                Some("test-key"),
                &Cancellation::default(),
                &base,
            )
            .await
            .unwrap_err();
            assert!(error.contains("Originalen er bevaret"));
            request.recv().unwrap();
            task.join().unwrap();
        }
    }
    #[tokio::test]
    async fn text_preserves_prompt_excludes_reasoning_and_returns_only_final_content() {
        let (base, request, task) = server(
            200,
            r#"{"choices":[{"finish_reason":"stop","message":{"content":"Nej, den 12.","reasoning":"private analysis"}}]}"#,
            Duration::ZERO,
        );
        let p = plan();
        let text = completion_at(
            &client(),
            &p,
            "nej den 12",
            Some("test-key"),
            &Cancellation::default(),
            &base,
        )
        .await
        .unwrap();
        assert_eq!(text, "Nej, den 12.");
        let raw = request.recv().unwrap();
        let split = raw.windows(4).position(|s| s == b"\r\n\r\n").unwrap() + 4;
        let body: Value = serde_json::from_slice(&raw[split..]).unwrap();
        assert_eq!(body["model"], "openai/gpt-oss-120b");
        assert_eq!(body["messages"][0]["content"], p.system_prompt);
        assert_eq!(body["include_reasoning"], false);
        assert_eq!(body["reasoning_effort"], "low");
        assert!(body.get("tools").is_none());
        task.join().unwrap();
    }
    #[tokio::test]
    async fn incomplete_text_is_an_error_for_original_fallback() {
        let (base, request, task) = server(
            200,
            r#"{"choices":[{"finish_reason":"length","message":{"content":"incomplete"}}]}"#,
            Duration::ZERO,
        );
        let error = completion_at(
            &client(),
            &plan(),
            "Original",
            Some("test-key"),
            &Cancellation::default(),
            &base,
        )
        .await
        .unwrap_err();
        assert!(error.contains("Originalen er bevaret"));
        request.recv().unwrap();
        task.join().unwrap();
    }
    #[tokio::test]
    async fn authentication_and_rate_errors_never_echo_upstream_data() {
        for (code, expected) in [(401, "Adgang afvist"), (429, "grænse")] {
            let (base, request, task) =
                server(code, r#"{"error":"PRIVATE-DATA test-key"}"#, Duration::ZERO);
            let error = models_at(&client(), "test-key", &Cancellation::default(), &base)
                .await
                .unwrap_err();
            assert!(error.contains(expected));
            assert!(!error.contains("PRIVATE-DATA"));
            assert!(!error.contains("test-key"));
            request.recv().unwrap();
            task.join().unwrap();
        }
    }
    #[tokio::test]
    async fn access_check_uses_get_and_sends_no_audio_or_transcript() {
        let (base, request, task) = server(
            200,
            r#"{"data":[{"id":"whisper-large-v3-turbo"},{"id":"openai/gpt-oss-120b"}]}"#,
            Duration::ZERO,
        );
        let models = models_at(
            &client(),
            "test-key",
            &Cancellation::default(),
            &(base + "/models"),
        )
        .await
        .unwrap();
        assert_eq!(models.len(), 2);
        let bytes = request.recv().unwrap();
        let raw = String::from_utf8_lossy(&bytes);
        assert!(raw.starts_with("GET /models "));
        assert!(raw.ends_with("\r\n\r\n"));
        task.join().unwrap();
    }
    #[tokio::test]
    async fn accepted_cancellation_drops_pending_cloud_request() {
        let (base, request, task) = server(200, r#"{"data":[]}"#, Duration::from_millis(300));
        let cancel = Cancellation::default();
        let work_cancel = cancel.clone();
        let work =
            tokio::spawn(
                async move { models_at(&client(), "test-key", &work_cancel, &base).await },
            );
        for attempt in 0..100 {
            if request.try_recv().is_ok() {
                break;
            }
            assert!(attempt < 99, "request did not arrive");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(cancel.cancel());
        assert_eq!(work.await.unwrap().unwrap_err(), "Annulleret.");
        task.join().unwrap();
    }
}
