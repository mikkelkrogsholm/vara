use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Recording,
    Recognizing,
    Processing,
    Committing,
    Completed,
    Cancelled,
    Failed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub generation: String,
    pub origin: Destination,
    pub plan: Plan,
    pub stage: Stage,
    pub created_at: i64,
    pub recovery_id: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CancelOutcome {
    Cancelled,
    AlreadyCancelled,
    AlreadyCommitted,
}
impl Session {
    pub fn new(origin: Destination, plan: Plan, now: i64) -> Self {
        Self {
            generation: uuid::Uuid::new_v4().to_string(),
            origin,
            plan,
            stage: Stage::Recording,
            created_at: now,
            recovery_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    pub fn advance(&mut self, from: Stage, to: Stage) -> Result<()> {
        if self.stage == Stage::Cancelled {
            return Err(Error::Cancelled);
        }
        if self.stage != from {
            return Err(Error::Invalid(
                "Dikteringen er allerede gået videre.".into(),
            ));
        }
        let allowed = matches!(
            (from, to),
            (Stage::Recording, Stage::Recognizing)
                | (Stage::Recognizing, Stage::Processing)
                | (Stage::Recognizing, Stage::Committing)
                | (Stage::Processing, Stage::Committing)
                | (Stage::Committing, Stage::Completed)
        );
        if !allowed {
            return Err(Error::Invalid("Ugyldigt trin i dikteringen.".into()));
        }
        self.stage = to;
        Ok(())
    }
    pub fn cancel(&mut self) -> CancelOutcome {
        match self.stage {
            Stage::Committing | Stage::Completed => CancelOutcome::AlreadyCommitted,
            Stage::Cancelled => CancelOutcome::AlreadyCancelled,
            _ => {
                self.stage = Stage::Cancelled;
                CancelOutcome::Cancelled
            }
        }
    }
}
