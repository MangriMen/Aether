use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
/// A manifest containing information about a mod loader's versions
pub struct ModdedManifestDto {
    /// The game versions the mod loader supports
    pub game_versions: Vec<ModdedVersionDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
///  A game version of Minecraft
pub struct ModdedVersionDto {
    /// The minecraft version ID
    pub id: String,
    /// Whether the release is stable or not
    pub stable: bool,
    /// A map that contains loader versions for the game version
    pub loaders: Vec<ModdedLoaderVersionDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
/// A version of a Minecraft mod loader
pub struct ModdedLoaderVersionDto {
    /// The version ID of the loader
    pub id: String,
    /// The URL of the version's manifest
    pub url: String,
    /// Whether the loader is stable or not
    pub stable: bool,
}

impl From<aether_core::features::minecraft::modded::Manifest> for ModdedManifestDto {
    fn from(value: aether_core::features::minecraft::modded::Manifest) -> Self {
        Self {
            game_versions: value.game_versions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::minecraft::modded::Version> for ModdedVersionDto {
    fn from(value: aether_core::features::minecraft::modded::Version) -> Self {
        Self {
            id: value.id,
            stable: value.stable,
            loaders: value.loaders.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::minecraft::modded::LoaderVersion> for ModdedLoaderVersionDto {
    fn from(value: aether_core::features::minecraft::modded::LoaderVersion) -> Self {
        Self {
            id: value.id,
            url: value.url,
            stable: value.stable,
        }
    }
}
