use aether_core::features::events::EventError;
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

use super::CauseDto;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventErrorDto {
    NotInitialized,
    NoLoadingBar { id: Uuid },
    SerializeError { cause: CauseDto },
    StorageError { cause: CauseDto },
}

impl From<&EventError> for EventErrorDto {
    fn from(value: &EventError) -> Self {
        match value {
            EventError::NotInitialized => Self::NotInitialized,
            EventError::NoLoadingBar(id) => Self::NoLoadingBar { id: *id },
            EventError::SerializeError(err) => Self::SerializeError {
                cause: CauseDto::unknown(err),
            },
            EventError::StorageError(err) => Self::StorageError {
                cause: CauseDto::unknown(err),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_serialize_serialize_error_with_an_unknown_cause() {
        let dto = EventErrorDto::from(&EventError::SerializeError(anyhow::anyhow!("boom")));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "SERIALIZE_ERROR", "payload": {"cause": {"type": "unknown", "details": "boom"}}})
        );
    }
}
