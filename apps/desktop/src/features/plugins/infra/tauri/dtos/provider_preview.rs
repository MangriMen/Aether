use serde::{Deserialize, Serialize};
use specta::Type;

use super::{PluginCapabilitiesDto, PluginManifestPreviewDto};

/// Generic provider-agnostic plugin preview for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderPluginPreviewDto {
    pub tag: String,
    pub manifest: PluginManifestPreviewDto,
    /// As the launcher would register them (handlers, flags derived from them, platform).
    pub capabilities: Option<PluginCapabilitiesDto>,
    pub archive_sha256: String,
}

impl From<aether_core::features::plugins::PluginReleasePreview> for ProviderPluginPreviewDto {
    fn from(value: aether_core::features::plugins::PluginReleasePreview) -> Self {
        Self {
            tag: value.tag,
            manifest: PluginManifestPreviewDto::from(value.manifest),
            capabilities: value.capabilities.map(PluginCapabilitiesDto::from),
            archive_sha256: value.archive_sha256,
        }
    }
}

/// The releases of a plugin that can be installed, and the one to start from.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginReleaseListingDto {
    pub owner: String,
    pub repo: String,
    pub releases: Vec<ProviderReleaseInfoDto>,
    pub default_tag: String,
}

impl From<aether_core::features::plugins::PluginReleaseListing> for PluginReleaseListingDto {
    fn from(value: aether_core::features::plugins::PluginReleaseListing) -> Self {
        Self {
            owner: value.owner,
            repo: value.repo,
            releases: value
                .releases
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
            default_tag: value.default_tag,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderReleaseInfoDto {
    pub tag_name: String,
    pub version: String,
    pub is_prerelease: bool,
    pub published_at: String,
    pub html_url: String,
    pub download_url: String,
}
