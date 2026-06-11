use serde::{Deserialize, Serialize};

use crate::v0::ProviderIdDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseContentParamsDto {
    pub content_id: String,
    pub content_version: Option<String>,
    pub provider_id: ProviderIdDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomicInstallParamsDto {
    pub base: BaseContentParamsDto,
    pub instance_id: String,
    pub game_version: String,
    pub loader: Option<String>,
    pub content_type: crate::v0::ContentTypeDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInstallParamsDto {
    pub base: BaseContentParamsDto,
}
