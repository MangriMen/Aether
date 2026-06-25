use serde::{Deserialize, Serialize};
use specta::Type;

use super::GitHubReleaseInfoDto;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GitHubPluginPreviewDto {
    pub owner: String,
    pub repo: String,
    pub manifest: Option<PluginManifestPreviewDto>,
    pub capabilities: Option<String>,
    pub releases: Vec<GitHubReleaseInfoDto>,
}

impl From<aether_core::features::plugins::GitHubPluginPreview> for GitHubPluginPreviewDto {
    fn from(value: aether_core::features::plugins::GitHubPluginPreview) -> Self {
        Self {
            owner: value.owner,
            repo: value.repo,
            manifest: value.manifest.map(Into::into),
            capabilities: value.capabilities,
            releases: value.releases.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginManifestPreviewDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub api_version: Option<String>,
}

impl From<aether_core::features::plugins::PluginManifestPreview> for PluginManifestPreviewDto {
    fn from(value: aether_core::features::plugins::PluginManifestPreview) -> Self {
        Self {
            id: value.id,
            name: value.name,
            version: value.version,
            description: value.description,
            authors: value.authors,
            license: value.license,
            api_version: value.api_version,
        }
    }
}
