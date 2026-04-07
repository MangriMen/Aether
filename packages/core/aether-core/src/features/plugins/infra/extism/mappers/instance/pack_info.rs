use aether_core_plugin_api::v0::PackInfoDto;

use crate::features::instance::PackInfo;

impl From<PackInfoDto> for PackInfo {
    fn from(value: PackInfoDto) -> Self {
        Self {
            plugin_id: value.plugin_id,
            modpack_id: value.modpack_id,
            version: value.version,
        }
    }
}
