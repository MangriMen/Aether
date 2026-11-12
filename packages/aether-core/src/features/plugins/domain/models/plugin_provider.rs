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

/// Metadata returned by a provider for the preview screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPluginPreview {
    /// Owner / namespace within the source (e.g., GitHub owner).
    pub owner: String,
    /// Repository / project name within the source.
    pub repo: String,
    /// Parsed manifest if available.
    pub manifest: Option<PluginManifestPreview>,
    /// Raw capabilities JSON if available.
    pub capabilities: Option<String>,
    /// Available releases/versions.
    pub releases: Vec<ProviderReleaseInfo>,
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

/// One release as the user is shown it before installing: the release itself, and the manifest
/// and capabilities the provider published beside the archive.
#[derive(Debug, Clone)]
pub struct ProviderReleasePreview {
    pub release: ProviderReleaseInfo,
    pub manifest: Option<PluginManifestPreview>,
    pub capabilities: Option<String>,
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
