use aether_core::features::minecraft::ModLoader;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "index.ts")]
pub enum ModLoaderDto {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl From<ModLoader> for ModLoaderDto {
    fn from(value: ModLoader) -> Self {
        match value {
            ModLoader::Vanilla => Self::Vanilla,
            ModLoader::Forge => Self::Forge,
            ModLoader::Fabric => Self::Fabric,
            ModLoader::Quilt => Self::Quilt,
            ModLoader::NeoForge => Self::NeoForge,
        }
    }
}

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
