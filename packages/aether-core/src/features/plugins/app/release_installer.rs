use std::sync::Arc;

use crate::features::plugins::{
    ExtractedPlugin, PluginError, PluginRegistry, PluginSettings, PluginSettingsStorage,
    PluginSource, PluginSourceStorage, PluginStorage, PluginVerificationStorage,
};

use super::record_plugin_verification;

/// Puts a downloaded release in place of the plugin's installed files, or of nothing, together
/// with what the core keeps about it. It happens whole or not at all: on a failure at any step
/// the installed version, its settings, trust record and source are as they were, or a plugin
/// that was not installed is not left half there.
pub(super) struct ReleaseInstaller {
    pub(super) registry: Arc<PluginRegistry>,
    pub(super) storage: Arc<dyn PluginStorage>,
    pub(super) source_storage: Arc<dyn PluginSourceStorage>,
    pub(super) verification_storage: Arc<dyn PluginVerificationStorage>,
    pub(super) settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl ReleaseInstaller {
    /// `known_installed` is for a caller that has the plugin installed already; otherwise it is
    /// found out under the lock, once nothing else can remove or replace it. A failure of a first
    /// install removes what it wrote instead of putting files back.
    pub(super) async fn install(
        &self,
        extracted: ExtractedPlugin,
        source: &PluginSource,
        known_installed: bool,
    ) -> Result<(), PluginError> {
        let plugin_id = extracted.plugin_id.clone();

        // The settings live in the plugin's directory, which the new version replaces. Held
        // until the old directory is dropped or back, so that a grant made meanwhile is neither
        // saved over nor lost with the directory.
        let _settings_lock = self.registry.lock_settings(&plugin_id).await;
        let settings = self.settings_storage.get(&plugin_id).await?;
        let was_installed = known_installed || self.storage.get(&plugin_id).await.is_ok();

        self.storage.replace(extracted).await?;

        match self.complete(&plugin_id, settings, source).await {
            Ok(()) => {
                // The new version is complete without it: a leftover is only disk space.
                if let Err(err) = self.storage.discard_replaced(&plugin_id).await {
                    tracing::warn!(
                        "Previous version of plugin '{plugin_id}' was left behind: {err}"
                    );
                }
                Ok(())
            }
            Err(err) => {
                self.undo(&plugin_id, was_installed).await;
                Err(err)
            }
        }
    }

    async fn undo(&self, plugin_id: &str, was_installed: bool) {
        if was_installed {
            if let Err(err) = self.storage.restore_replaced(plugin_id).await {
                tracing::error!(
                    "Plugin '{plugin_id}' could not be put back after a failed install: {err}"
                );
            }
        } else if let Err(err) = self.storage.remove(plugin_id).await {
            tracing::error!(
                "Plugin '{plugin_id}' could not be removed after a failed install: {err}"
            );
        }
    }

    /// Writes the core's files about the new version into its directory. The source goes last:
    /// it names the installed release, and must not name the new one before the rest is there.
    async fn complete(
        &self,
        plugin_id: &str,
        settings: Option<PluginSettings>,
        source: &PluginSource,
    ) -> Result<(), PluginError> {
        if let Some(mut settings) = settings {
            settings.force_enabled_at_api_version = None;
            self.settings_storage.upsert(plugin_id, &settings).await?;
        }

        // Trust is established on the bytes this release just delivered; the old record stays
        // with the old directory.
        record_plugin_verification(
            self.storage.as_ref(),
            self.verification_storage.as_ref(),
            plugin_id,
            source.to_source_type(),
        )
        .await?;

        self.source_storage.save(plugin_id, source).await
    }
}
