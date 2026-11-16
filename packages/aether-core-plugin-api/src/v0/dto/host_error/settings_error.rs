use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum SettingsErrorDto {
    NotFound,
    Storage(String),
}
