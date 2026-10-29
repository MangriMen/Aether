use aether_core_plugin_api::v0::PackInfoDto;

use crate::features::instance::{InvalidId, PackInfo};

impl TryFrom<PackInfoDto> for PackInfo {
    type Error = InvalidId;

    fn try_from(value: PackInfoDto) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_id: value.provider_id.try_into()?,
            modpack_id: value.modpack_id,
            version_id: value.version_id,
        })
    }
}

impl From<PackInfo> for PackInfoDto {
    fn from(value: PackInfo) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            modpack_id: value.modpack_id,
            version_id: value.version_id,
        }
    }
}
