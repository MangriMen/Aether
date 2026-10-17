use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
/// A manifest containing information about a mod loader's versions
pub struct ModdedManifestDto {
    /// The game versions the mod loader supports
    pub game_versions: Vec<ModdedVersionDto>,
    /// Groups of game versions that share the same loader versions
    pub version_groups: Vec<ModdedVersionGroupDto>,
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
    /// The group whose loader versions apply to this game version, instead of `loaders`
    #[serde(rename = "versionGroup")]
    pub version_group: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
/// A group of game versions that share the same loader versions
pub struct ModdedVersionGroupDto {
    /// The version group ID
    pub id: String,
    /// The loader versions of the group
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
            version_groups: value.version_groups.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::minecraft::modded::Version> for ModdedVersionDto {
    fn from(value: aether_core::features::minecraft::modded::Version) -> Self {
        Self {
            id: value.id,
            stable: value.stable,
            loaders: value.loaders.into_iter().map(Into::into).collect(),
            version_group: value.version_group,
        }
    }
}

impl From<aether_core::features::minecraft::modded::VersionGroup> for ModdedVersionGroupDto {
    fn from(value: aether_core::features::minecraft::modded::VersionGroup) -> Self {
        Self {
            id: value.id,
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
