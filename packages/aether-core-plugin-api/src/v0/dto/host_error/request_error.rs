use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RequestErrorDto {
    Acquire(String),
    RequestSend(String),
    Middleware(String),
    Hash { actual: String, expected: String },
    Json(String),
    Toml(String),
    Parse(String),
    Status { status: u16, url: String },
    TooLarge { limit: u64 },
    Io { path: String, details: String },
}
