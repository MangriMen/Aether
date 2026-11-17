use aether_core_plugin_api::v0::ContentTypeDto;

use super::UnknownValue;
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

const CONTENT_TYPES: [&str; 5] = ["modpack", "mod", "dataPack", "resourcePack", "shaderPack"];

impl TryFrom<ContentTypeDto> for ContentType {
    type Error = UnknownValue;

    fn try_from(value: ContentTypeDto) -> Result<Self, Self::Error> {
        match value {
            ContentTypeDto::Modpack => Ok(Self::Modpack),
            ContentTypeDto::Mod => Ok(Self::Mod),
            ContentTypeDto::DataPack => Ok(Self::DataPack),
            ContentTypeDto::ResourcePack => Ok(Self::ResourcePack),
            ContentTypeDto::ShaderPack => Ok(Self::ShaderPack),
            ContentTypeDto::Unknown => Err(UnknownValue {
                kind: "content type",
                expected: &CONTENT_TYPES,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_a_content_type_the_api_does_not_know() {
        assert_eq!(
            ContentType::try_from(ContentTypeDto::Unknown),
            Err(UnknownValue {
                kind: "content type",
                expected: &CONTENT_TYPES,
            })
        );
    }

    #[test]
    fn should_keep_a_known_content_type() {
        assert_eq!(
            ContentType::try_from(ContentTypeDto::DataPack),
            Ok(ContentType::DataPack)
        );
    }
}
