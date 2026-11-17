use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum EventErrorDto {
    NotInitialized,
    NoLoadingBar { id: String },
    SerializeError(String),
    StorageError(String),
}
