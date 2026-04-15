use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "index.ts", rename = "ModdedManifestDto")]
/// A manifest containing information about a mod loader's versions
pub struct ManifestDto {
    /// The game versions the mod loader supports
    pub game_versions: Vec<VersionDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "index.ts", rename = "ModdedVersionDto")]
///  A game version of Minecraft
pub struct VersionDto {
    /// The minecraft version ID
    pub id: String,
    /// Whether the release is stable or not
    pub stable: bool,
    /// A map that contains loader versions for the game version
    pub loaders: Vec<LoaderVersionDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "index.ts", rename = "ModdedLoaderVersionDto")]
/// A version of a Minecraft mod loader
pub struct LoaderVersionDto {
    /// The version ID of the loader
    pub id: String,
    /// The URL of the version's manifest
    pub url: String,
    /// Whether the loader is stable or not
    pub stable: bool,
}

impl From<aether_core::features::minecraft::modded::Manifest> for ManifestDto {
    fn from(value: aether_core::features::minecraft::modded::Manifest) -> Self {
        Self {
            game_versions: value.game_versions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::minecraft::modded::Version> for VersionDto {
    fn from(value: aether_core::features::minecraft::modded::Version) -> Self {
        Self {
            id: value.id,
            stable: value.stable,
            loaders: value.loaders.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::minecraft::modded::LoaderVersion> for LoaderVersionDto {
    fn from(value: aether_core::features::minecraft::modded::LoaderVersion) -> Self {
        Self {
            id: value.id,
            url: value.url,
            stable: value.stable,
        }
    }
}
