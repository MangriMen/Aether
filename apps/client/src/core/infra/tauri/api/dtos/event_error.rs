use aether_core::features::events::EventError;
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventErrorDto {
    NotInitialized,
    NoLoadingBar { id: Uuid },
    SerializeError { details: String },
    StorageError { details: String },
}

impl From<&EventError> for EventErrorDto {
    fn from(value: &EventError) -> Self {
        match value {
            EventError::NotInitialized => Self::NotInitialized,
            EventError::NoLoadingBar(id) => Self::NoLoadingBar { id: *id },
            EventError::SerializeError(err) => Self::SerializeError {
                details: err.to_string(),
            },
            EventError::StorageError(err) => Self::StorageError {
                details: err.to_string(),
            },
        }
    }
}
