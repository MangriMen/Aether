use serde::{Deserialize, Serialize};
use specta::Type;

use super::{PluginCapabilitiesDto, PluginManifestPreviewDto};

/// Generic provider-agnostic plugin preview for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderPluginPreviewDto {
    pub owner: String,
    pub repo: String,
    pub manifest: Option<PluginManifestPreviewDto>,
    pub capabilities: Option<PluginCapabilitiesDto>,
    pub releases: Vec<ProviderReleaseInfoDto>,
}

impl From<aether_core::features::plugins::ProviderPluginPreview> for ProviderPluginPreviewDto {
    fn from(value: aether_core::features::plugins::ProviderPluginPreview) -> Self {
        let capabilities = value
            .capabilities
            .as_deref()
            .and_then(|raw| serde_json::from_str::<PluginCapabilitiesDto>(raw).ok());

        Self {
            owner: value.owner,
            repo: value.repo,
            manifest: value.manifest.map(|m| PluginManifestPreviewDto {
                id: m.id,
                name: m.name,
                version: m.version,
                description: m.description,
                authors: m.authors,
                license: m.license,
                api_version: m.api_version,
            }),
            capabilities,
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
