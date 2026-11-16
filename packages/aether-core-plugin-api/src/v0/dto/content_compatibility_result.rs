use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentCompatibilityResultDto {
    pub is_compatible: bool,
}
