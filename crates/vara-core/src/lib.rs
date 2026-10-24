//! Platform-neutral Vara contracts. No Tauri, microphone, window, secret-store or ML runtime dependency.
pub mod model;
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
    Policy(String),
    #[error("Der er allerede en diktering i gang.")]
    Busy,
    #[error("Dikteringen er annulleret eller forældet.")]
    Cancelled,
    #[error("Optagelsen er udløbet og kan ikke genprøves.")]
    Expired,
    #[error("Historikken kunne ikke gemmes. Teksten er stadig tilgængelig.")]
    Storage(#[source] rusqlite::Error),
    #[error("Data kunne ikke læses: {0}")]
    Data(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
