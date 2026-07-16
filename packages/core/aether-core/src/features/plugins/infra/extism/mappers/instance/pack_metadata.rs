use aether_core_plugin_api::v0::PackMetadataDto;

use crate::features::instance::PackMetadata;

impl From<PackMetadataDto> for PackMetadata {
    fn from(value: PackMetadataDto) -> Self {
        Self {
            name: value.name,
            game_version: value.game_version,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version,
            pack_info: value.pack_info.map(Into::into),
        }
    }
}
