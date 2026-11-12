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

/// The releases a provider lists for a plugin. What a release holds is read from its archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPluginPreview {
    /// Owner / namespace within the source (e.g., GitHub owner).
    pub owner: String,
    /// Repository / project name within the source.
    pub repo: String,
    /// Available releases/versions.
    pub releases: Vec<ProviderReleaseInfo>,
}

/// One release as the user is shown it before installing, read from its archive: what it asks
/// for is what the archive holds, nothing a publisher states beside it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReleasePreview {
    pub owner: String,
    pub repo: String,
    /// Every release the provider lists, so that another can be picked.
    pub releases: Vec<ProviderReleaseInfo>,
    /// The tag of the release this preview is of.
    pub tag: String,
    pub manifest: PluginManifestPreview,
    /// The archive's `capabilities.json`, as written, when it has one.
    pub capabilities: Option<String>,
    /// Lowercase hex SHA-256 of the archive that was read. The install is given it back and
    /// refuses an archive that hashes differently.
    pub archive_sha256: String,
    /// Size of the archive, in bytes.
    pub size: u64,
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
