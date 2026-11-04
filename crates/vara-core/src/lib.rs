//! Platform-neutral Vara contracts. No Tauri, microphone, window, secret-store or ML runtime dependency.
pub mod message;
pub use message::UserMessage;
pub mod model;
pub mod transcription;
pub use transcription::*;
pub mod session;
pub mod store;
pub use model::*;
pub use session::*;
pub use store::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    InvalidMessage(UserMessage),
    #[error("{0}")]
    PolicyMessage(UserMessage),
    #[error("{0}")]
    Policy(String),
    #[error("A dictation is already in progress.")]
    Busy,
    #[error("The dictation was cancelled or is outdated.")]
    Cancelled,
    #[error("The recording has expired and cannot be retried.")]
    Expired,
    #[error("History could not be saved. The text is still available.")]
    Storage(#[source] rusqlite::Error),
    #[error("The file library could not be read or saved. The original files are preserved.")]
    FileStorage(#[source] rusqlite::Error),
    #[error("Data could not be read: {0}")]
    Data(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Explicit transport; Display remains compatible with local Rust clients.
    pub fn wire(&self) -> String {
        let code = match self {
            Self::Busy => "session.busy",
            Self::Cancelled => "session.cancelled",
            Self::Expired => "session.expired",
            Self::Storage(error) => return storage_message("storage.history", error).wire(),
            Self::FileStorage(error) => return storage_message("storage.files", error).wire(),
            Self::Data(error) => {
                return UserMessage::new("storage.data")
                    .caused_by(
                        UserMessage::new("storage.json_location")
                            .number("line", error.line() as i64)
                            .number("column", error.column() as i64),
                    )
                    .wire();
            }
            Self::Invalid(text) | Self::Policy(text) => return text.clone(),
            Self::InvalidMessage(message) | Self::PolicyMessage(message) => return message.wire(),
        };
        UserMessage::new(code).wire()
    }
}

fn storage_message(code: &str, error: &rusqlite::Error) -> UserMessage {
    // SQLite's free-form message may contain SQL or user data. Keep only its numeric code.
    let cause = match error {
        rusqlite::Error::SqliteFailure(detail, _) => {
            UserMessage::new("storage.sqlite_code").number("code", detail.extended_code.into())
        }
        _ => UserMessage::new("storage.sqlite_other"),
    };
    UserMessage::new(code).caused_by(cause)
}

#[cfg(test)]
mod error_message_tests {
    use super::*;
    #[test]
    fn validation_and_storage_preserve_transport_and_safe_cause() {
        let mut settings = Settings::default();
        settings.retention_days = Some(0);
        let error = settings.validate().unwrap_err();
        assert_eq!(
            UserMessage::decode(&error.wire()).unwrap().code,
            "settings.retention_days"
        );
        assert!(!error.to_string().contains(message::PREFIX));
        let db = rusqlite::Connection::open_in_memory().unwrap();
        let error = Error::Storage(
            db.execute("INSERT INTO missing VALUES ('private')", [])
                .unwrap_err(),
        );
        let wire = error.wire();
        assert!(!wire.contains("private"));
        assert_eq!(UserMessage::decode(&wire).unwrap().code, "storage.history");
    }
}
