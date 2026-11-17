use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum SettingsErrorDto {
    NotFound,
    Storage(String),
}
