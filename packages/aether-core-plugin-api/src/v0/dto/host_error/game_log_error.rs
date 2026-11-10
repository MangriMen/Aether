use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum GameLogErrorDto {
    InvalidSource { source_id: String },
    SourceNotFound { source_id: String },
    TooLarge { source_id: String, limit: u64 },
    Io(String),
}
