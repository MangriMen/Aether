use aether_core::features::game_log::GameLogError;
use serde::Serialize;
use specta::Type;

use super::CauseDto;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameLogErrorDto {
    InvalidSource { source_id: String },
    SourceNotFound { source_id: String },
    TooLarge { source_id: String, limit: u32 },
    Io { cause: CauseDto },
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
            GameLogError::Io(err) => Self::Io { cause: err.into() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::shared::io::IoError;
    use serde_json::json;

    #[test]
    fn should_serialize_io_with_an_io_cause() {
        let err = GameLogError::Io(IoError::IoError(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )));

        assert_eq!(
            serde_json::to_value(GameLogErrorDto::from(&err)).unwrap(),
            json!({"code": "IO", "payload": {"cause": {"type": "io", "kind": "notFound"}}})
        );
    }
}
