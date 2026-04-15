use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "index.ts")]
/// The version type
pub enum VersionTypeDto {
    /// A major version, which is stable for all players to use
    Release,
    /// An experimental version, which is unstable and used for feature previews and beta testing
    Snapshot,
    /// The oldest versions before the game was released
    OldAlpha,
    /// Early versions of the game
    OldBeta,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "index.ts")]
/// A game version of Minecraft
pub struct VersionDto {
    /// A unique identifier of the version
    pub id: String,
    #[serde(rename = "type")]
    /// The release type of the version
    pub type_: VersionTypeDto,
    /// A link to additional information about the version
    pub url: String,
    /// The latest time a file in this version was updated
    pub time: DateTime<Utc>,
    /// The time this version was released
    pub release_time: DateTime<Utc>,
    /// The SHA1 hash of the additional information about the version
    pub sha1: String,
    /// Whether the version supports the latest player safety features
    pub compliance_level: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    /// (Modrinth Provided) The SHA1 hash of the original unmodified Minecraft versions JSON
    pub original_sha1: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "index.ts")]
/// The latest snapshot and release of the game
pub struct LatestVersionDto {
    /// The version id of the latest release
    pub release: String,
    /// The version id of the latest snapshot
    pub snapshot: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "index.ts")]
/// Data of all game versions of Minecraft
pub struct VersionManifestDto {
    /// A struct containing the latest snapshot and release of the game
    pub latest: LatestVersionDto,
    /// A list of game versions of Minecraft
    pub versions: Vec<VersionDto>,
}

impl From<aether_core::features::minecraft::vanilla::VersionManifest> for VersionManifestDto {
    fn from(value: aether_core::features::minecraft::vanilla::VersionManifest) -> Self {
        Self {
            latest: LatestVersionDto {
                release: value.latest.release,
                snapshot: value.latest.snapshot,
            },
            versions: value.versions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::minecraft::vanilla::Version> for VersionDto {
    fn from(value: aether_core::features::minecraft::vanilla::Version) -> Self {
        Self {
            id: value.id,
            type_: value.type_.into(),
            url: value.url,
            time: value.time,
            release_time: value.release_time,
            sha1: value.sha1,
            compliance_level: value.compliance_level,
            original_sha1: value.original_sha1,
        }
    }
}

impl From<aether_core::features::minecraft::vanilla::VersionType> for VersionTypeDto {
    fn from(value: aether_core::features::minecraft::vanilla::VersionType) -> Self {
        match value {
            aether_core::features::minecraft::vanilla::VersionType::Release => Self::Release,
            aether_core::features::minecraft::vanilla::VersionType::Snapshot => Self::Snapshot,
            aether_core::features::minecraft::vanilla::VersionType::OldAlpha => Self::OldAlpha,
            aether_core::features::minecraft::vanilla::VersionType::OldBeta => Self::OldBeta,
        }
    }
}
