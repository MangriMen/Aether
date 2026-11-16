use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::ProviderIdDto;

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoDto {
    pub provider_id: ProviderIdDto,
    pub modpack_id: String,
    pub version_id: String,
}
