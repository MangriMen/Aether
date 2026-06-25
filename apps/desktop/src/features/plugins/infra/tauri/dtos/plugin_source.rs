use serde::{Deserialize, Serialize};
use specta::Type;

use super::ProviderReleaseInfoDto;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum PluginSourceDto {
    GitHub {
        owner: String,
        repo: String,
        current_tag: String,
        current_version: String,
    },
    Local,
}

impl From<aether_core::features::plugins::PluginSource> for PluginSourceDto {
    fn from(value: aether_core::features::plugins::PluginSource) -> Self {
        match value {
            aether_core::features::plugins::PluginSource::GitHub {
                owner,
                repo,
                current_tag,
                current_version,
            } => Self::GitHub {
                owner,
                repo,
                current_tag,
                current_version,
            },
            aether_core::features::plugins::PluginSource::Local => Self::Local,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GitHubReleaseInfoDto {
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

impl From<aether_core::features::plugins::GitHubReleaseInfo> for GitHubReleaseInfoDto {
    fn from(value: aether_core::features::plugins::GitHubReleaseInfo) -> Self {
        Self {
            tag_name: value.tag_name,
            version: value.version,
            is_prerelease: value.is_prerelease,
            published_at: value.published_at,
            html_url: value.html_url,
            zip_download_url: value.zip_download_url,
            manifest_download_url: value.manifest_download_url,
            capabilities_download_url: value.capabilities_download_url,
            plugin_id: value.plugin_id,
        }
    }
}

/// Provider-agnostic plugin update info.
/// Uses `ProviderReleaseInfoDto` for releases so it works with any source type.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginUpdateInfoDto {
    pub current_version: String,
    pub current_tag: String,
    pub latest_version: Option<String>,
    pub latest_tag: Option<String>,
    pub has_update: bool,
    pub all_releases: Vec<ProviderReleaseInfoDto>,
}

impl From<aether_core::features::plugins::ProviderUpdateInfo> for PluginUpdateInfoDto {
    fn from(value: aether_core::features::plugins::ProviderUpdateInfo) -> Self {
        Self {
            current_version: value.current_version,
            current_tag: value.current_tag,
            latest_version: value.latest_version,
            latest_tag: value.latest_tag,
            has_update: value.has_update,
            all_releases: value
                .all_releases
                .into_iter()
                .map(|r| ProviderReleaseInfoDto {
                    tag_name: r.tag_name,
                    version: r.version,
                    is_prerelease: r.is_prerelease,
                    published_at: r.published_at,
                    html_url: r.html_url,
                    download_url: r.download_url,
                })
                .collect(),
        }
    }
}
