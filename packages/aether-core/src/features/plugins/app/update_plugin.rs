use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginExtractor, PluginProviderError, PluginRegistry, PluginSettings,
    PluginSettingsStorage, PluginSource, PluginSourceStorage, PluginStorage,
    PluginVerificationStorage,
};

use super::ports::UpdatePluginUseCasePort;
use super::{PluginProviderFactory, record_plugin_verification, write_bytes_to_temp_file};

/// Updates a plugin to a specific version (or latest) using the provider factory.
/// Works with any provider type (GitHub, Modrinth, etc.).
///
/// The new release replaces the plugin's files, and the user's settings of it carry over — all
/// but a force-enable, which confirms one specific build. An update that fails at any
/// step leaves the installed version, its settings, trust record and source as they were.
pub(crate) struct UpdatePluginUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_storage: Arc<dyn PluginStorage>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    provider_factory: Arc<PluginProviderFactory>,
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
    ) -> Self {
        Self {
            plugin_registry,
            plugin_extractor,
            plugin_storage,
            plugin_source_storage,
            plugin_verification_storage,
            plugin_settings_storage,
            provider_factory,
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

        // The settings live in the plugin's directory, which the new version replaces. Held
        // until the old directory is dropped or back, so that a grant made meanwhile is neither
        // saved over nor lost with the directory.
        let _settings_lock = self.plugin_registry.lock_settings(plugin_id).await;
        let settings = self.plugin_settings_storage.get(plugin_id).await?;

        self.plugin_storage.replace(extracted).await?;

        match self
            .complete_replacement(plugin_id, settings, &updated_source)
            .await
        {
            Ok(()) => {
                // The new version is complete without it: a leftover is only disk space.
                if let Err(err) = self.plugin_storage.discard_replaced(plugin_id).await {
                    tracing::warn!(
                        "Previous version of plugin '{plugin_id}' was left behind: {err}"
                    );
                }
                Ok(())
            }
            Err(err) => {
                if let Err(restore_err) = self.plugin_storage.restore_replaced(plugin_id).await {
                    tracing::error!(
                        "Plugin '{plugin_id}' could not be put back after a failed update: \
                         {restore_err}"
                    );
                }
                Err(err)
            }
        }
    }

    /// Writes the core's files about the new version into its directory. The source goes last:
    /// it names the installed release, and must not name the new one before the rest is there.
    async fn complete_replacement(
        &self,
        plugin_id: &str,
        settings: Option<PluginSettings>,
        updated_source: &PluginSource,
    ) -> Result<(), PluginError> {
        if let Some(mut settings) = settings {
            settings.force_enabled_at_api_version = None;
            self.plugin_settings_storage
                .upsert(plugin_id, &settings)
                .await?;
        }

        // Trust is established on the bytes this release just delivered; the old record stays
        // with the old directory.
        record_plugin_verification(
            self.plugin_storage.as_ref(),
            self.plugin_verification_storage.as_ref(),
            plugin_id,
            updated_source.to_source_type(),
        )
        .await?;

        self.plugin_source_storage
            .save(plugin_id, updated_source)
            .await
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
