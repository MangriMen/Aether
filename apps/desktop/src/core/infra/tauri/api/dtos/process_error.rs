use aether_core::features::{game_log::GameLogError, process::ProcessError};
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessErrorDto {
    KillError { id: String },
    WaitError { id: String },
    Io { details: String },
    InvalidLogSource { source_id: String },
    LogSourceNotFound { source_id: String },
    LogTooLarge { source_id: String, limit: u32 },
}

impl From<&ProcessError> for ProcessErrorDto {
    fn from(value: &ProcessError) -> Self {
        match value {
            ProcessError::KillError { id } => Self::KillError { id: id.clone() },
            ProcessError::WaitError { id } => Self::WaitError { id: id.clone() },
            ProcessError::Io(err) => Self::Io {
                details: err.to_string(),
            },
            ProcessError::GameLog(err) => err.into(),
        }
    }
}

impl From<&GameLogError> for ProcessErrorDto {
    fn from(value: &GameLogError) -> Self {
        match value {
            GameLogError::InvalidSource { source_id } => Self::InvalidLogSource {
                source_id: source_id.clone(),
            },
            GameLogError::SourceNotFound { source_id } => Self::LogSourceNotFound {
                source_id: source_id.clone(),
            },
            GameLogError::TooLarge { source_id, limit } => Self::LogTooLarge {
                source_id: source_id.clone(),
                limit: u32::try_from(*limit).unwrap_or(u32::MAX),
            },
            GameLogError::Io(err) => Self::Io {
                details: err.to_string(),
            },
        }
    }
}
