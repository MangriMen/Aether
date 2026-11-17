use aether_core_plugin_api::v0::ModLoaderDto;

use super::UnknownValue;
use crate::features::minecraft::ModLoader;

impl TryFrom<ModLoaderDto> for ModLoader {
    type Error = UnknownValue;

    fn try_from(value: ModLoaderDto) -> Result<Self, Self::Error> {
        match value {
            ModLoaderDto::Vanilla => Ok(Self::Vanilla),
            ModLoaderDto::Forge => Ok(Self::Forge),
            ModLoaderDto::Fabric => Ok(Self::Fabric),
            ModLoaderDto::Quilt => Ok(Self::Quilt),
            ModLoaderDto::NeoForge => Ok(Self::NeoForge),
            ModLoaderDto::Unknown => Err(UnknownValue { kind: "mod loader" }),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_a_loader_the_api_does_not_know() {
        assert_eq!(
            ModLoader::try_from(ModLoaderDto::Unknown),
            Err(UnknownValue { kind: "mod loader" })
        );
    }

    #[test]
    fn should_keep_a_known_loader() {
        assert_eq!(
            ModLoader::try_from(ModLoaderDto::NeoForge),
            Ok(ModLoader::NeoForge)
        );
    }
}
