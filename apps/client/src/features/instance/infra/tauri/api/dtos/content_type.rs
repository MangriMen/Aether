use aether_core::features::instance::ContentType;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum ContentTypeDto {
    Modpack,
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
}

impl From<ContentTypeDto> for ContentType {
    fn from(value: ContentTypeDto) -> Self {
        match value {
            ContentTypeDto::Modpack => Self::Modpack,
            ContentTypeDto::Mod => Self::Mod,
            ContentTypeDto::DataPack => Self::DataPack,
            ContentTypeDto::ResourcePack => Self::ResourcePack,
            ContentTypeDto::ShaderPack => Self::ShaderPack,
        }
    }
}

impl From<ContentType> for ContentTypeDto {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::Modpack => Self::Modpack,
            ContentType::Mod => Self::Mod,
            ContentType::DataPack => Self::DataPack,
            ContentType::ResourcePack => Self::ResourcePack,
            ContentType::ShaderPack => Self::ShaderPack,
        }
    }
}
