use aether_core_plugin_api::v0::ContentTypeDto;

use crate::features::instance::ContentType;

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
