use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginExtractor, PluginManifestPreview, PluginReleasePreview, PluginSourceType,
};

use super::ports::PreviewPluginReleaseUseCasePort;
use super::release_archive::download_release_archive;
use super::{PluginProviderFactory, ReleaseArchiveCache};

/// Describes a release from its archive, not from what the publisher states beside it.
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
        let downloaded = download_release_archive(
            &self.provider_factory,
            &self.archive_cache,
            &source_type,
            identifier,
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
