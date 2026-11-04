use aether_core_plugin_api::v0::PackInfoDto;

use crate::features::instance::{InvalidContentId, InvalidId, PackInfo};

/// What is wrong with a pack info a plugin handed over.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPackInfo {
    #[error(transparent)]
    Provider(#[from] InvalidId),

    #[error(transparent)]
    ContentId(#[from] InvalidContentId),
}

impl TryFrom<PackInfoDto> for PackInfo {
    type Error = InvalidPackInfo;

    fn try_from(value: PackInfoDto) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_id: value.provider_id.try_into()?,
            modpack_id: value.modpack_id.try_into()?,
            version_id: value.version_id,
        })
    }
}

impl From<PackInfo> for PackInfoDto {
    fn from(value: PackInfo) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            modpack_id: value.modpack_id.into(),
            version_id: value.version_id,
        }
    }
}
