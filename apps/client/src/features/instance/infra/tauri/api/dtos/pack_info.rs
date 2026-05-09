use aether_core::features::instance::PackInfo;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::ProviderIdDto;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoDto {
    pub provider_id: ProviderIdDto,
    pub modpack_id: String,
    pub version: String,
}

impl From<PackInfoDto> for PackInfo {
    fn from(value: PackInfoDto) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            modpack_id: value.modpack_id,
            version_id: value.version,
        }
    }
}

impl From<PackInfo> for PackInfoDto {
    fn from(value: PackInfo) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            modpack_id: value.modpack_id,
            version: value.version_id,
        }
    }
}
