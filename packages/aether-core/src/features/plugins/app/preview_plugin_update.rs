use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    PluginError, PluginExtractor, PluginSourceStorage, PluginStorage, PluginUpdateDiff,
    PluginUpdatePreview,
};

use super::ports::PreviewPluginUpdateUseCasePort;
use super::preview_plugin_release::read_release;
use super::release_archive::{ReleaseSource, recorded_source};
use super::{PluginProviderFactory, ReleaseArchiveCache};

/// Shows a release of an installed plugin before the update to it: what its archive asks for, and
/// how that differs from the version installed now. The update is then given the archive's hash,
/// so what it installs is the archive shown here.
pub(crate) struct PreviewPluginUpdateUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_storage: Arc<dyn PluginStorage>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    provider_factory: Arc<PluginProviderFactory>,
    archive_cache: Arc<ReleaseArchiveCache>,
}

impl PreviewPluginUpdateUseCase {
    pub(crate) fn new(
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        plugin_source_storage: Arc<dyn PluginSourceStorage>,
        provider_factory: Arc<PluginProviderFactory>,
        archive_cache: Arc<ReleaseArchiveCache>,
    ) -> Self {
        Self {
            plugin_extractor,
            plugin_storage,
            plugin_source_storage,
            provider_factory,
            archive_cache,
        }
    }

    pub(crate) async fn execute(
        &self,
        plugin_id: &PluginId,
        tag: &str,
    ) -> Result<PluginUpdatePreview, PluginError> {
        let (source_type, identifier) =
            recorded_source(self.plugin_source_storage.as_ref(), plugin_id).await?;
        let installed = self.plugin_storage.get(plugin_id).await?;

        let (release, manifest) = read_release(
            self.plugin_extractor.as_ref(),
            &self.provider_factory,
            &self.archive_cache,
            &source_type,
            ReleaseSource::Recorded(&identifier),
            tag,
        )
        .await?;

        // The update refuses such an archive: it is not offered as one.
        if &manifest.metadata.id != plugin_id {
            return Err(PluginError::IdMismatch {
                plugin_id: plugin_id.clone(),
                archive_id: manifest.metadata.id,
            });
        }

        let diff = PluginUpdateDiff::between(
            &installed.manifest,
            installed.capabilities.as_ref(),
            &manifest,
            release.capabilities.as_ref(),
        );

        Ok(PluginUpdatePreview {
            release,
            installed_version: installed.manifest.metadata.version.to_string(),
            diff,
        })
    }
}

#[async_trait]
impl PreviewPluginUpdateUseCasePort for PreviewPluginUpdateUseCase {
    async fn execute(
        &self,
        plugin_id: &PluginId,
        tag: &str,
    ) -> Result<PluginUpdatePreview, PluginError> {
        self.execute(plugin_id, tag).await
    }
}
