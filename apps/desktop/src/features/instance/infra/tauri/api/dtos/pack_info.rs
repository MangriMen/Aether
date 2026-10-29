use aether_core::features::instance::{InstanceError, PackInfo};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::infra::ProviderIdDto;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoDto {
    pub provider_id: ProviderIdDto,
    pub modpack_id: String,
    pub version: String,
}

impl TryFrom<PackInfoDto> for PackInfo {
    type Error = InstanceError;

    fn try_from(value: PackInfoDto) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_id: value.provider_id.try_into()?,
            modpack_id: value.modpack_id,
            version_id: value.version,
        })
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
