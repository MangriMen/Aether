use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoDto {
    pub plugin_id: String,
    pub modpack_id: String,
    pub version: String,
}
