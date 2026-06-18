use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum ProcessErrorDto {
    KillError { id: String },
    WaitError { id: String },
    Io(String),
}
