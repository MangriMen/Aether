use aether_core::features::instance::PackInfo;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoDto {
    pub plugin_id: String,
    pub modpack_id: String,
    pub version: String,
}

impl From<PackInfoDto> for PackInfo {
    fn from(value: PackInfoDto) -> Self {
        Self {
            plugin_id: value.plugin_id,
            modpack_id: value.modpack_id,
            version: value.version,
        }
    }
}

impl From<PackInfo> for PackInfoDto {
    fn from(value: PackInfo) -> Self {
        Self {
            plugin_id: value.plugin_id,
            modpack_id: value.modpack_id,
            version: value.version,
        }
    }
}
