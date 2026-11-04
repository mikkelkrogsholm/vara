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
        _ => return Err(vara_core::UserMessage::new("cli.unknown").wire()),
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
    let code = match provider {
        Provider::ClaudeCli => "cli.missing_claude",
        Provider::CodexCli => "cli.missing_codex",
        Provider::GeminiCli => "cli.missing_gemini",
        _ => "cli.unknown",
    };
    Err(vara_core::UserMessage::new(code).wire())
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
    let mut child = command.spawn().map_err(|e| {
        vara_core::UserMessage::new("cli.start")
            .caused_by(vara_core::UserMessage::io_cause(&e))
            .wire()
    })?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or(vara_core::UserMessage::new("cli.no_input").wire())?;
    if let Some(input) = input {
        stdin.write_all(input.as_bytes()).await.map_err(|e| {
            vara_core::UserMessage::new("cli.input_rejected")
                .caused_by(vara_core::UserMessage::io_cause(&e))
                .wire()
        })?;
    }
    drop(stdin);
    let mut stdout = child
        .stdout
        .take()
        .ok_or(vara_core::UserMessage::new("cli.no_output").wire())?
        .take(2_000_001);
    let mut bytes = Vec::new();
    let start = std::time::Instant::now();
    let work = async {
        stdout.read_to_end(&mut bytes).await.map_err(|e| {
            vara_core::UserMessage::new("cli.output_read")
                .caused_by(vara_core::UserMessage::io_cause(&e))
                .wire()
        })?;
        if bytes.len() > 2_000_000 {
            return Err(vara_core::UserMessage::new("cli.output_too_large").wire());
        }
        let status = child.wait().await.map_err(|e| {
            vara_core::UserMessage::new("cli.wait")
                .caused_by(vara_core::UserMessage::io_cause(&e))
                .wire()
        })?;
        if !status.success() {
            return Err(vara_core::UserMessage::new("cli.rejected")
                .caused_by(crate::process_cause(&status))
                .wire());
        }
        String::from_utf8(bytes).map_err(|_| vara_core::UserMessage::new("cli.invalid_text").wire())
    };
    tokio::pin!(work);
    loop {
        tokio::select! {r=&mut work=>{cancel.check()?;return r;},_=tokio::time::sleep(std::time::Duration::from_millis(50))=>{cancel.check()?;if start.elapsed()>std::time::Duration::from_secs(seconds){return Err(vara_core::UserMessage::new("cli.timeout").wire());}}}
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
        return Status {
            provider,
            installed: true,
            supported: false,
            detail: vara_core::UserMessage::new("cli.codex_unsupported").wire(),
        };
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
            vara_core::UserMessage::new("cli.supported").wire()
        } else {
            let mut message = vara_core::UserMessage::new("cli.unsupported");
            if let Some(cause) = output
                .as_ref()
                .err()
                .and_then(|e| vara_core::UserMessage::decode(e))
            {
                message = message.caused_by(cause);
            }
            message.wire()
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
        .map_err(|e| {
            vara_core::UserMessage::new("cli.workdir")
                .caused_by(vara_core::UserMessage::io_cause(&e))
                .wire()
        })?;
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
            std::fs::write(&settings,serde_json::to_vec(&serde_json::json!({"tools":{"core":[],"discoveryCommand":"","callCommand":""},"mcp":{"allowed":["__vara_no_mcp__"]},"hooksConfig":{"enabled":false},"context":{"fileName":"__vara_no_context__","includeDirectories":[]}})).unwrap()).map_err(|_|vara_core::UserMessage::new("cli.config").wire())?;
            std::fs::write(&system, &plan.system_prompt).map_err(|e| {
                vara_core::UserMessage::new("cli.system_prompt")
                    .caused_by(vara_core::UserMessage::io_cause(&e))
                    .wire()
            })?;
            command
                .env("GEMINI_CLI_SYSTEM_SETTINGS_PATH", settings)
                .env("GEMINI_SYSTEM_MD", system)
                .args([
                    "--prompt",
                    "Process only the transcription from stdin. Return only the finished text.",
                    "--extensions",
                    "none",
                    "--output-format",
                    "json",
                ]);
            if plan.engine.model != "default" {
                command.args(["--model", &plan.engine.model]);
            }
        }
        _ => return Err(vara_core::UserMessage::new("cli.isolation").wire()),
    }
    let input = format!(
        "<transskription>\n{}\n</transskription>",
        text.replace("</transskription>", "< /transskription>")
    );
    let out = output(command, Some(input), cancel, 90).await?;
    if plan.engine.provider == Provider::GeminiCli {
        let value: serde_json::Value = serde_json::from_str(&out)
            .map_err(|_| vara_core::UserMessage::new("cli.gemini_invalid").wire())?;
        if value.get("error").is_some() {
            return Err(vara_core::UserMessage::new("cli.gemini_failed").wire());
        }
        return value["response"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| vara_core::UserMessage::new("cli.gemini_empty").wire());
    }
    Ok(out)
}
