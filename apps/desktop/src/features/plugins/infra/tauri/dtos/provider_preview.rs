use aether_core::features::plugins::{PluginCapabilities, infra::PluginCapabilitiesV1};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{PluginCapabilitiesDto, PluginManifestPreviewDto};

/// Generic provider-agnostic plugin preview for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderPluginPreviewDto {
    pub owner: String,
    pub repo: String,
    pub releases: Vec<ProviderReleaseInfoDto>,
    /// The tag the manifest, capabilities and hash below were read from.
    pub tag: String,
    pub manifest: PluginManifestPreviewDto,
    pub capabilities: Option<PluginCapabilitiesDto>,
    /// Hash of the archive that was read; the install holds the download to it.
    pub archive_sha256: String,
    #[specta(type = f64)]
    pub size: u64,
}

impl From<aether_core::features::plugins::PluginReleasePreview> for ProviderPluginPreviewDto {
    fn from(value: aether_core::features::plugins::PluginReleasePreview) -> Self {
        // Read the raw file the way loading the plugin does, so the preview shows what the
        // launcher would register (handlers, flags derived from them, platform).
        let capabilities = value
            .capabilities
            .as_deref()
            .and_then(|raw| serde_json::from_str::<PluginCapabilitiesV1>(raw).ok())
            .and_then(|v1| {
                PluginCapabilities::try_from(v1)
                    .inspect_err(|err| {
                        tracing::warn!("Dropping the capabilities of a plugin preview: {err}");
                    })
                    .ok()
            })
            .map(PluginCapabilitiesDto::from);

        Self {
            owner: value.owner,
            repo: value.repo,
            tag: value.tag,
            manifest: PluginManifestPreviewDto::from(value.manifest),
            capabilities,
            archive_sha256: value.archive_sha256,
            size: value.size,
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
