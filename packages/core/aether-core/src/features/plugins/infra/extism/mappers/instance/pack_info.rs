use aether_core_plugin_api::v0::PackInfoDto;

use crate::features::instance::PackInfo;

impl From<PackInfoDto> for PackInfo {
    fn from(value: PackInfoDto) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            modpack_id: value.modpack_id,
            version_id: value.version,
        }
    }
}
