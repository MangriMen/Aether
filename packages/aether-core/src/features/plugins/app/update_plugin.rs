use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginExtractor, PluginProviderError, PluginRegistry, PluginSettingsStorage,
    PluginSource, PluginSourceStorage, PluginStorage, PluginSyncService, PluginVerificationStorage,
};

use super::ports::UpdatePluginUseCasePort;
use super::release_installer::ReleaseInstaller;
use super::{PluginProviderFactory, write_bytes_to_temp_file};

/// Updates a plugin to a specific version (or latest) using the provider factory.
/// Works with any provider type (GitHub, Modrinth, etc.).
///
/// The new release replaces the plugin's files, and the user's settings of it carry over — all
/// but a force-enable, which confirms one specific build. An update that fails at any
/// step leaves the installed version, its settings, trust record and source as they were.
///
/// Once replaced, the plugin is synced, so that the registry holds the new build and a plugin
/// that was enabled runs it. The update is done by then, so a failed sync is logged, not returned.
pub(crate) struct UpdatePluginUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    installer: ReleaseInstaller,
    provider_factory: Arc<PluginProviderFactory>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl UpdatePluginUseCase {
    // One port per collaborator, wired once in the DI container; a parameter struct would only
    // repeat the field list.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        plugin_source_storage: Arc<dyn PluginSourceStorage>,
        plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        provider_factory: Arc<PluginProviderFactory>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_extractor,
            plugin_source_storage: plugin_source_storage.clone(),
            installer: ReleaseInstaller {
                registry: plugin_registry,
                storage: plugin_storage,
                source_storage: plugin_source_storage,
                verification_storage: plugin_verification_storage,
                settings_storage: plugin_settings_storage,
            },
            provider_factory,
            sync_plugins_service,
        }
    }

    pub(crate) async fn execute(
        &self,
        plugin_id: &str,
        target_tag: Option<&str>,
    ) -> Result<(), PluginError> {
        let source = self
            .plugin_source_storage
            .get(plugin_id)
            .await?
            .ok_or_else(|| PluginProviderError::NoRemoteSource {
                plugin_id: plugin_id.to_string(),
            })?;

        let source_type = source.to_source_type();
        let identifier = source_identifier(&source);

        let provider = self
            .provider_factory
            .get_provider(&source_type)
            .ok_or_else(|| PluginProviderError::NoRemoteSource {
                plugin_id: plugin_id.to_string(),
            })?;

        let tag = if let Some(t) = target_tag {
            t.to_string()
        } else {
            let current_tag = source.current_tag().unwrap_or_default();
            let current_version = source_version(&source);
            let updates = provider
                .fetch_latest_version(&identifier, current_tag, current_version)
                .await?;

            updates
                .latest_tag
                .ok_or_else(|| PluginProviderError::ProviderFetchError {
                    source_type: source_type.clone(),
                    details: "No releases found".to_string(),
                })?
        };

        // Fetch preview to get all releases with download URLs
        let preview = provider.fetch_preview(&identifier).await?;

        let release = preview
            .releases
            .into_iter()
            .find(|r| r.tag_name == tag)
            .ok_or_else(|| PluginProviderError::ProviderNoAssets {
                source_type: source_type.clone(),
            })?;

        // Download zip to temp file, reuse PluginExtractor
        let zip_bytes = provider.download_plugin(&release.download_url).await?;
        let temp_file = write_bytes_to_temp_file(&zip_bytes)?;

        // Unpacked before anything is removed: an archive that cannot be read, or that is of
        // another plugin, leaves the installed version in place.
        let extracted = self.plugin_extractor.extract(temp_file.path()).await?;
        if extracted.plugin_id != plugin_id {
            return Err(PluginError::IdMismatch {
                plugin_id: plugin_id.to_string(),
                archive_id: extracted.plugin_id,
            });
        }

        let updated_source = PluginSource::Remote {
            source_type,
            identifier,
            current_tag: release.tag_name,
            current_version: release.version,
        };

        self.installer
            .install(extracted, &updated_source, true)
            .await?;

        // Once replaced, so that enabling the new build, which can take seconds, holds nothing.
        if let Err(err) = self.sync_plugins_service.execute().await {
            log::warn!("Plugin '{plugin_id}' was updated but the sync after it failed: {err}");
        }
        Ok(())
    }
}

/// Build a provider identifier from a `PluginSource`.
fn source_identifier(source: &PluginSource) -> String {
    match source {
        PluginSource::Remote { identifier, .. } => identifier.clone(),
        PluginSource::Local => String::new(),
    }
}

/// Build a provider version from a `PluginSource`.
fn source_version(source: &PluginSource) -> &str {
    match source {
        PluginSource::Remote {
            current_version, ..
        } => current_version,
        PluginSource::Local => "",
    }
}

#[async_trait]
impl UpdatePluginUseCasePort for UpdatePluginUseCase {
    async fn execute(&self, plugin_id: &str, target_tag: Option<&str>) -> Result<(), PluginError> {
        self.execute(plugin_id, target_tag).await
    }
}
