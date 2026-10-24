//! Optional installed text CLIs. No shell interpolation, project context or model-selected tools.
use crate::{Cancellation, Result};
use serde::Serialize;
use std::{path::PathBuf, process::Stdio};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use vara_core::{Provider, TextPlan};
#[derive(Serialize)]
pub struct Status {
    pub provider: Provider,
    pub installed: bool,
    pub supported: bool,
    pub detail: String,
}
fn binary(provider: Provider) -> Result<PathBuf> {
    let name = match provider {
        Provider::ClaudeCli => "claude",
        Provider::CodexCli => "codex",
        Provider::GeminiCli => "gemini",
        _ => return Err("Ukendt CLI.".into()),
    };
    let mut paths: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    #[cfg(unix)]
    {
        paths.extend([
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
        ]);
        if let Some(home) = std::env::var_os("HOME") {
            paths.push(PathBuf::from(home).join(".local/bin"));
        }
    }
    for dir in paths {
        let path = dir.join(name);
        if path.is_file() {
            return Ok(path);
        }
        #[cfg(windows)]
        {
            let path = dir.join(format!("{name}.exe"));
            if path.is_file() {
                return Ok(path);
            }
        }
    }
    Err(format!(
        "{name} er ikke installeret. Installer og log ind i værktøjet uden for Vara."
    ))
}
async fn output(
    mut command: Command,
    input: Option<String>,
    cancel: &Cancellation,
    seconds: u64,
) -> Result<String> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| "CLI kunne ikke starte.")?;
    let mut stdin = child.stdin.take().ok_or("CLI mangler tekstinput.")?;
    if let Some(input) = input {
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|_| "CLI afviste tekstinput.")?;
    }
    drop(stdin);
    let mut stdout = child
        .stdout
        .take()
        .ok_or("CLI mangler tekstoutput.")?
        .take(2_000_001);
    let mut bytes = Vec::new();
    let start = std::time::Instant::now();
    let work = async {
        stdout
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| "CLI-svaret kunne ikke læses.")?;
        if bytes.len() > 2_000_000 {
            return Err("CLI-svaret er for stort.".into());
        }
        let status = child
            .wait()
            .await
            .map_err(|_| "CLI kunne ikke afsluttes.")?;
        if !status.success() {
            return Err("CLI afviste forespørgslen. Kontroller login og model i værktøjet.".into());
        }
        String::from_utf8(bytes).map_err(|_| "CLI returnerede ugyldig tekst.".into())
    };
    tokio::pin!(work);
    loop {
        tokio::select! {r=&mut work=>{cancel.check()?;return r;},_=tokio::time::sleep(std::time::Duration::from_millis(50))=>{cancel.check()?;if start.elapsed()>std::time::Duration::from_secs(seconds){return Err("CLI nåede tidsgrænsen. Originalen er bevaret.".into());}}}
    }
}
pub async fn status(provider: Provider) -> Status {
    let path = match binary(provider) {
        Ok(p) => p,
        Err(detail) => {
            return Status {
                provider,
                installed: false,
                supported: false,
                detail,
            };
        }
    };
    if provider == Provider::CodexCli {
        return Status{provider,installed:true,supported:false,detail:"Installeret. Denne Codex-kontrakt har ingen verificeret tilstand uden værktøjer; tekstbehandling er derfor utilgængelig.".into()};
    }
    let mut command = Command::new(path);
    command.arg(if provider == Provider::ClaudeCli {
        "--help"
    } else {
        "--version"
    });
    let output = output(command, None, &Cancellation::default(), 10).await;
    let supported = match (&output, provider) {
        (Ok(h), Provider::ClaudeCli) => [
            "--tools",
            "--strict-mcp-config",
            "--setting-sources",
            "--disable-slash-commands",
            "--no-session-persistence",
            "--output-format",
        ]
        .iter()
        .all(|flag| h.contains(flag)),
        (Ok(v), Provider::GeminiCli) => v.trim() == "0.29.5",
        _ => false,
    };
    Status {
        provider,
        installed: true,
        supported,
        detail: if supported {
            "Tekstkontrakten er understøttet. Login og modeladgang kontrolleres ved forespørgslen."
                .into()
        } else {
            "Installeret version har ingen verificeret isoleret tekstkontrakt.".into()
        },
    }
}
pub async fn text(plan: &TextPlan, text: &str, cancel: &Cancellation) -> Result<String> {
    let status = status(plan.engine.provider).await;
    if !status.supported {
        return Err(status.detail);
    }
    let dir = tempfile::Builder::new()
        .prefix("vara-text-")
        .tempdir()
        .map_err(|_| "CLI-arbejdsmappe kunne ikke oprettes.")?;
    let mut command = Command::new(binary(plan.engine.provider)?);
    command.current_dir(dir.path());
    match plan.engine.provider {
        Provider::ClaudeCli => {
            command.args([
                "--print",
                "--output-format",
                "text",
                "--tools",
                "",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
                "--setting-sources",
                "",
                "--settings",
                "{\"disableAllHooks\":true}",
                "--disable-slash-commands",
                "--no-session-persistence",
                "--system-prompt",
                &plan.system_prompt,
            ]);
            if plan.engine.model != "default" {
                command.args(["--model", &plan.engine.model]);
            }
        }
        Provider::GeminiCli => {
            // Schema and tool allowlist behavior verified against installed 0.29.5.
            let settings = dir.path().join("settings.json");
            let system = dir.path().join("system.md");
            std::fs::write(&settings,serde_json::to_vec(&serde_json::json!({"tools":{"core":[],"discoveryCommand":"","callCommand":""},"mcp":{"allowed":["__vara_no_mcp__"]},"hooksConfig":{"enabled":false},"context":{"fileName":"__vara_no_context__","includeDirectories":[]}})).unwrap()).map_err(|_|"CLI-konfiguration kunne ikke oprettes.")?;
            std::fs::write(&system, &plan.system_prompt)
                .map_err(|_| "Systemprompt kunne ikke oprettes.")?;
            command
                .env("GEMINI_CLI_SYSTEM_SETTINGS_PATH", settings)
                .env("GEMINI_SYSTEM_MD", system)
                .args([
                    "--prompt",
                    "Bearbejd kun transskriptionen fra stdin. Returnér kun færdig tekst.",
                    "--extensions",
                    "none",
                    "--output-format",
                    "json",
                ]);
            if plan.engine.model != "default" {
                command.args(["--model", &plan.engine.model]);
            }
        }
        _ => return Err("CLI mangler en isoleret tekstkontrakt.".into()),
    }
    let input = format!(
        "<transskription>\n{}\n</transskription>",
        text.replace("</transskription>", "< /transskription>")
    );
    let out = output(command, Some(input), cancel, 90).await?;
    if plan.engine.provider == Provider::GeminiCli {
        let value: serde_json::Value =
            serde_json::from_str(&out).map_err(|_| "Gemini CLI returnerede et ugyldigt svar.")?;
        if value.get("error").is_some() {
            return Err("Gemini CLI kunne ikke behandle teksten.".into());
        }
        return value["response"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Gemini CLI returnerede ingen tekst.".into());
    }
    Ok(out)
}
