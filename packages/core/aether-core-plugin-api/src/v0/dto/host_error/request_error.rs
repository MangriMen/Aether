use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum RequestErrorDto {
    Acquire(String),
    RequestSend(String),
    Middleware(String),
    Hash { actual: String, expected: String },
    Json(String),
    Toml(String),
    Parse(String),
}
