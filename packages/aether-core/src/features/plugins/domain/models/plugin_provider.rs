use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::PluginManifestPreview;

/// Identifies the source type for a plugin provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSourceType {
    GitHub,
    Local,
}

impl FromStr for PluginSourceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" | "git_hub" => Ok(Self::GitHub),
            "local" => Ok(Self::Local),
            _ => Err(()),
        }
    }
}

impl fmt::Display for PluginSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::Local => write!(f, "local"),
        }
    }
}

/// The releases a provider lists; their contents come from the archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPluginPreview {
    /// Owner / namespace within the source (e.g., GitHub owner).
    pub owner: String,
    /// Repository / project name within the source.
    pub repo: String,
    /// Available releases/versions.
    pub releases: Vec<ProviderReleaseInfo>,
}

/// A release as shown before install, read from its archive rather than from publisher metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReleasePreview {
    pub owner: String,
    pub repo: String,
    pub releases: Vec<ProviderReleaseInfo>,
    pub tag: String,
    pub manifest: PluginManifestPreview,
    pub capabilities: Option<String>,
    /// Lowercase hex SHA-256 of the archive that was read. The install is given it back and
    /// refuses an archive that hashes differently.
    pub archive_sha256: String,
}

/// Information about a single release from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderReleaseInfo {
    pub tag_name: String,
    pub version: String,
    pub is_prerelease: bool,
    pub published_at: String,
    pub html_url: String,
    pub download_url: String,
}

/// Result of checking for updates from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUpdateInfo {
    pub current_version: String,
    pub current_tag: String,
    pub latest_version: Option<String>,
    pub latest_tag: Option<String>,
    pub has_update: bool,
    pub all_releases: Vec<ProviderReleaseInfo>,
}

/// A registered provider as the user is offered it.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginProviderInfo {
    pub source_type: PluginSourceType,
    /// The provider's own name, shown as it is.
    pub name: String,
}
