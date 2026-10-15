use crate::features::plugins::PluginManifestPreview;

/// Preview of a GitHub plugin before installation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitHubPluginPreview {
    pub owner: String,
    pub repo: String,
    pub manifest: Option<PluginManifestPreview>,
    pub capabilities: Option<String>,
    pub releases: Vec<GitHubReleaseInfo>,
}

/// Information about a GitHub release that the frontend
/// can use to display update availability and version choices.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitHubReleaseInfo {
    pub tag_name: String,
    pub version: String,
    pub is_prerelease: bool,
    pub published_at: String,
    pub html_url: String,
    pub zip_download_url: String,
    pub manifest_download_url: Option<String>,
    pub capabilities_download_url: Option<String>,
    pub plugin_id: Option<String>,
}

/// DTO for plugin update check results from GitHub.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginUpdateInfo {
    pub current_version: String,
    pub current_tag: String,
    pub latest_version: Option<String>,
    pub latest_tag: Option<String>,
    pub has_update: bool,
    pub all_releases: Vec<GitHubReleaseInfo>,
}
