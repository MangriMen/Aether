use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum EventErrorDto {
    NotInitialized,
    NoLoadingBar { id: String },
    SerializeError(String),
    StorageError(String),
}
