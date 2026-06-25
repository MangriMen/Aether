use serde::{Deserialize, Serialize};

use super::PluginSourceType;

/// Describes where a plugin was originally installed from,
/// enabling update checks and version switching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum PluginSource {
    /// Plugin was installed from a GitHub release.
    GitHub {
        owner: String,
        repo: String,
        current_tag: String,
        current_version: String,
    },
    /// Plugin was installed from a local file/archive.
    /// No update support available.
    Local,
}

impl PluginSource {
    /// Returns the full GitHub repository name (`owner/repo`) if applicable.
    pub fn github_repo(&self) -> Option<(&str, &str)> {
        match self {
            PluginSource::GitHub { owner, repo, .. } => Some((owner, repo)),
            PluginSource::Local => None,
        }
    }

    pub fn current_tag(&self) -> Option<&str> {
        match self {
            PluginSource::GitHub { current_tag, .. } => Some(current_tag),
            PluginSource::Local => None,
        }
    }

    /// Returns the provider source type for this plugin source.
    pub fn to_source_type(&self) -> PluginSourceType {
        match self {
            PluginSource::GitHub { .. } => PluginSourceType::GitHub,
            PluginSource::Local => PluginSourceType::Local,
        }
    }
}

/// Preview of a GitHub plugin before installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPluginPreview {
    pub owner: String,
    pub repo: String,
    pub manifest: Option<PluginManifestPreview>,
    pub capabilities: Option<String>,
    pub releases: Vec<GitHubReleaseInfo>,
}

/// Lightweight manifest info for preview (no runtime/load details).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifestPreview {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub api_version: Option<String>,
}

/// Information about a GitHub release that the frontend
/// can use to display update availability and version choices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReleaseInfo {
    pub tag_name: String,
    pub version: String,
    pub is_prerelease: bool,
    pub published_at: String,
    pub html_url: String,
    pub zip_download_url: String,
    pub manifest_download_url: Option<String>,
    pub capabilities_download_url: Option<String>,
    pub plugin_id: Option<String>, // populated after inspecting manifest
}

/// DTO for plugin update check results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateInfo {
    pub current_version: String,
    pub current_tag: String,
    pub latest_version: Option<String>,
    pub latest_tag: Option<String>,
    pub has_update: bool,
    pub all_releases: Vec<GitHubReleaseInfo>,
}
