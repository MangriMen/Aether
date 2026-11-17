use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GameLogErrorDto {
    InvalidSource { source_id: String },
    SourceNotFound { source_id: String },
    TooLarge { source_id: String, limit: u64 },
    Io(String),
}
