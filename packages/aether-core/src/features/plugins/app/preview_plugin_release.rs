use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginExtractor, PluginManifestPreview, PluginProviderError, PluginReleasePreview,
    PluginSourceType,
};

use super::ports::PreviewPluginReleaseUseCasePort;
use super::release_archive::download_release_archive;
use super::{PluginProviderFactory, ReleaseArchiveCache};

/// Shows a release of a plugin as its archive describes it.
///
/// The archive is downloaded, hashed and read without being unpacked, so that what the user is
/// shown is what the archive asks for, and nothing a publisher states beside it. The install is
/// given the hash back and refuses an archive that hashes differently.
pub(crate) struct PreviewPluginReleaseUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    provider_factory: Arc<PluginProviderFactory>,
    archive_cache: Arc<ReleaseArchiveCache>,
}

impl PreviewPluginReleaseUseCase {
    pub(crate) fn new(
        plugin_extractor: Arc<dyn PluginExtractor>,
        provider_factory: Arc<PluginProviderFactory>,
        archive_cache: Arc<ReleaseArchiveCache>,
    ) -> Self {
        Self {
            plugin_extractor,
            provider_factory,
            archive_cache,
        }
    }

    pub(crate) async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
        tag: Option<String>,
    ) -> Result<PluginReleasePreview, PluginError> {
        let provider = self
            .provider_factory
            .get_provider(&source_type)
            .ok_or_else(|| PluginProviderError::ProviderFetchError {
                source_type: source_type.clone(),
                details: "No provider is registered for it".to_string(),
            })?;
        let identifier = provider.parse_identifier(identifier)?;

        let downloaded = download_release_archive(
            provider,
            &self.archive_cache,
            &source_type,
            &identifier,
            tag.as_deref(),
        )
        .await?;
        let metadata = self
            .plugin_extractor
            .read_metadata(downloaded.archive.path())
            .await?;

        Ok(PluginReleasePreview {
            owner: downloaded.listing.owner,
            repo: downloaded.listing.repo,
            releases: downloaded.listing.releases,
            tag: downloaded.release.tag_name,
            manifest: PluginManifestPreview::from(&metadata.manifest),
            capabilities: metadata.capabilities,
            archive_sha256: downloaded.archive.sha256().to_owned(),
            size: downloaded.archive.size(),
        })
    }
}

#[async_trait]
impl PreviewPluginReleaseUseCasePort for PreviewPluginReleaseUseCase {
    async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
        tag: Option<String>,
    ) -> Result<PluginReleasePreview, PluginError> {
        self.execute(source_type, identifier, tag).await
    }
}
