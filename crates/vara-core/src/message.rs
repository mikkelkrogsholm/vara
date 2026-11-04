//! Versioned runtime messages. Parameters are numeric so secrets and user content
//! cannot accidentally be passed into the transport or interpolated in the UI.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PREFIX: &str = "vara-message-v1:";
pub const CATALOG: &str = include_str!("../messages/runtime.json");

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserMessage {
    pub code: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<UserMessage>>,
}
impl UserMessage {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.into(),
            params: BTreeMap::new(),
            cause: None,
        }
    }
    pub fn number(mut self, name: &str, value: i64) -> Self {
        self.params.insert(name.into(), value);
        self
    }
    /// Capture only an OS error kind and numeric code, never paths or free-form text.
    pub fn io_cause(error: &std::io::Error) -> Self {
        use std::io::ErrorKind;
        let code = match error.kind() {
            ErrorKind::NotFound => "io.not_found",
            ErrorKind::PermissionDenied => "io.permission_denied",
            ErrorKind::TimedOut => "io.timed_out",
            ErrorKind::Interrupted => "io.interrupted",
            ErrorKind::AlreadyExists => "io.already_exists",
            ErrorKind::InvalidData => "io.invalid_data",
            ErrorKind::WriteZero | ErrorKind::StorageFull => "io.storage",
            _ => "io.other",
        };
        let mut cause = Self::new(code);
        if let Some(raw) = error.raw_os_error() {
            cause = cause.caused_by(Self::new("io.os_code").number("code", raw.into()));
        }
        cause
    }
    pub fn caused_by(mut self, cause: UserMessage) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }
    pub fn wire(&self) -> String {
        format!(
            "{PREFIX}{}",
            serde_json::to_string(self).expect("message serialization")
        )
    }
    /// Legacy text and other protocol versions are deliberately not classified.
    pub fn decode(value: &str) -> Option<Self> {
        serde_json::from_str(value.strip_prefix(PREFIX)?).ok()
    }
    pub fn format(&self, locale: &str) -> String {
        let catalog: serde_json::Value =
            serde_json::from_str(CATALOG).expect("valid bundled catalog");
        let locale = locale.to_ascii_lowercase().replace('_', "-");
        let base = locale.split('-').next().unwrap_or("en");
        let entry = catalog
            .get(&self.code)
            .or_else(|| catalog.get("runtime.unknown"));
        let mut text = entry
            .and_then(|e| {
                e.get(&locale)
                    .or_else(|| e.get(base))
                    .or_else(|| e.get("en"))
            })
            .and_then(|s| s.as_str())
            .unwrap_or("An unknown error occurred.")
            .to_owned();
        for (name, value) in &self.params {
            text = text.replace(&format!("{{{name}}}"), &value.to_string());
        }
        if let Some(cause) = &self.cause {
            text.push_str(": ");
            text.push_str(&cause.format(&locale));
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_format_fallback_and_legacy() {
        let msg = UserMessage::new("provider.http")
            .number("status", 503)
            .caused_by(UserMessage::new("provider.network"));
        assert_eq!(UserMessage::decode(&msg.wire()), Some(msg.clone()));
        assert!(msg.format("da-DK").contains("HTTP 503"));
        assert!(msg.format("en-GB").contains("Connection failed"));
        assert_eq!(
            UserMessage::new("future.code").format("en"),
            "An unknown error occurred."
        );
        assert!(UserMessage::decode("Annulleret.").is_none());
        assert!(UserMessage::decode("vara-message-v2:{}").is_none());
    }
    fn placeholders(value: &str) -> std::collections::BTreeSet<&str> {
        let mut names = std::collections::BTreeSet::new();
        let mut tail = value;
        while let Some(start) = tail.find('{') {
            tail = &tail[start + 1..];
            let end = tail.find('}').expect("closed placeholder");
            names.insert(&tail[..end]);
            tail = &tail[end + 1..];
        }
        names
    }
    #[test]
    fn origin_codes_exist_in_catalog() {
        let catalog: serde_json::Value = serde_json::from_str(CATALOG).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let marker = ["UserMessage::", "new(\""].concat();
        for krate in ["vara-core", "vara-engine", "vara-platform"] {
            for entry in std::fs::read_dir(root.join(krate).join("src")).unwrap() {
                let path = entry.unwrap().path();
                if path.file_name().is_some_and(|name| name == "message.rs") {
                    continue;
                }
                if path.extension().is_none_or(|ext| ext != "rs") {
                    continue;
                }
                let source = std::fs::read_to_string(&path).unwrap();
                for marker in [&marker[..], "M::new(\""] {
                    for part in source.split(marker).skip(1) {
                        let code = part.split('"').next().unwrap();
                        assert!(
                            catalog.get(code).is_some(),
                            "{}: missing {code}",
                            path.display()
                        );
                    }
                }
            }
        }
        // Codes chosen by matches rather than literal constructor arguments.
        for code in [
            "session.busy",
            "session.cancelled",
            "session.expired",
            "storage.history",
            "storage.files",
            "storage.data",
            "provider.timeout",
            "provider.connect",
            "provider.transport",
            "io.not_found",
            "io.permission_denied",
            "io.timed_out",
            "io.interrupted",
            "io.already_exists",
            "io.invalid_data",
            "io.storage",
            "io.other",
            "io.os_code",
            "cli.missing_claude",
            "cli.missing_codex",
            "cli.missing_gemini",
            "cli.unknown",
            "credential.timeout_mutation",
            "credential.timeout",
            "whisper.spectrogram_missing",
            "whisper.spectrogram_failed",
            "whisper.encode_incomplete",
            "whisper.decode_incomplete",
            "whisper.mel_bands",
            "whisper.thread_count",
            "whisper.invalid_text",
            "whisper.null_byte",
            "whisper.null_pointer",
            "whisper.no_samples",
            "whisper.sample_length",
            "whisper.half_sample",
            "whisper.init",
            "whisper.state",
            "whisper.spectrogram",
            "whisper.encode",
            "whisper.decode",
            "whisper.evaluation",
            "whisper.parameters",
            "whisper.utf8",
            "whisper.null",
            "whisper.samples",
            "loader.open",
            "loader.symbol",
            "loader.other",
        ] {
            assert!(catalog.get(code).is_some(), "missing dynamic code {code}");
        }
    }
    #[test]
    fn io_detail_keeps_kind_and_code_without_free_text() {
        let error = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "secret token /private/path",
        );
        let wire = UserMessage::new("media.source_open")
            .caused_by(UserMessage::io_cause(&error))
            .wire();
        assert!(!wire.contains("secret") && !wire.contains("/private/path"));
        assert!(wire.contains("io.permission_denied"));
        let error = std::io::Error::from_raw_os_error(13);
        assert!(UserMessage::io_cause(&error).wire().contains("\"code\":13"));
    }
    #[test]
    fn catalogs_have_equal_nonempty_locales() {
        let c: serde_json::Value = serde_json::from_str(CATALOG).unwrap();
        for (code, values) in c.as_object().unwrap() {
            assert_eq!(
                placeholders(values["da"].as_str().unwrap()),
                placeholders(values["en"].as_str().unwrap()),
                "{code}"
            );
            for lang in ["da", "en"] {
                assert!(
                    !values[lang].as_str().unwrap_or("").is_empty(),
                    "{code}/{lang}"
                );
            }
        }
    }
}

impl std::fmt::Display for UserMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format("en"))
    }
}
