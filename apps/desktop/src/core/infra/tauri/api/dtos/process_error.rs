use aether_core::features::process::ProcessError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessErrorDto {
    KillError { id: String },
    WaitError { id: String },
    Io { details: String },
}

impl From<&ProcessError> for ProcessErrorDto {
    fn from(value: &ProcessError) -> Self {
        match value {
            ProcessError::KillError { id } => Self::KillError { id: id.clone() },
            ProcessError::WaitError { id } => Self::WaitError { id: id.clone() },
            ProcessError::Io(err) => Self::Io {
                details: err.to_string(),
            },
        }
    }
}
