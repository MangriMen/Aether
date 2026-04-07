use aether_core_plugin_api::v0::ModLoaderDto;

use crate::features::minecraft::ModLoader;

impl From<ModLoaderDto> for ModLoader {
    fn from(value: ModLoaderDto) -> Self {
        match value {
            ModLoaderDto::Vanilla => Self::Vanilla,
            ModLoaderDto::Forge => Self::Forge,
            ModLoaderDto::Fabric => Self::Fabric,
            ModLoaderDto::Quilt => Self::Quilt,
            ModLoaderDto::NeoForge => Self::NeoForge,
        }
    }
}
