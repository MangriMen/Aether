use aether_core::features::game_log::GameLogError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameLogErrorDto {
    InvalidSource { source_id: String },
    SourceNotFound { source_id: String },
    TooLarge { source_id: String, limit: u32 },
    Io { details: String },
}

impl From<&GameLogError> for GameLogErrorDto {
    fn from(value: &GameLogError) -> Self {
        match value {
            GameLogError::InvalidSource { source_id } => Self::InvalidSource {
                source_id: source_id.clone(),
            },
            GameLogError::SourceNotFound { source_id } => Self::SourceNotFound {
                source_id: source_id.clone(),
            },
            GameLogError::TooLarge { source_id, limit } => Self::TooLarge {
                source_id: source_id.clone(),
                limit: u32::try_from(*limit).unwrap_or(u32::MAX),
            },
            GameLogError::Io(err) => Self::Io {
                details: err.to_string(),
            },
        }
    }
}
