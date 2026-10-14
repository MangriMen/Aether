use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    plugins::{
        PLUGIN_API_VERSION, PluginError, PluginLoaderRegistry, PluginRegistry,
        PluginSettingsStorage,
    },
    settings::SettingsStorage,
};

use super::{plugin_enabler::PluginEnabler, ports::ForceEnablePluginUseCasePort};

/// Force-enables a plugin by skipping the API version compatibility check
/// and persisting the user's choice in the plugin settings.
///
/// On next restart the plugin will auto-load if the API version hasn't
/// changed. If the launcher is updated, the flag is ignored and the user
/// must re-confirm.
pub(crate) struct ForceEnablePluginUseCase {
    enabler: PluginEnabler,
}

impl ForceEnablePluginUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_loader_registry: Arc<PluginLoaderRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        settings_storage: Arc<dyn SettingsStorage>,
    ) -> Self {
        Self {
            enabler: PluginEnabler {
                plugin_registry,
                plugin_loader_registry,
                plugin_settings_storage,
                settings_storage,
            },
        }
    }

    /// Bypasses the API version check and loads the plugin directly, then saves the
    /// force-enable flag in plugin settings with the current API version.
    pub(crate) async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        // Forcing skips the API version check, not the state: a loaded plugin stays as it is.
        let (previous, manifest) = self
            .enabler
            .plugin_registry
            .begin_loading(&plugin_id)
            .await?;

        if let Err(err) = self.enabler.check_requirements(&plugin_id, &manifest).await {
            self.enabler
                .plugin_registry
                .abandon_loading(&plugin_id, previous)
                .await?;
            return Err(err);
        }

        self.enabler.load(&plugin_id, &manifest).await?;
        self.save_force_enabled_flag(&plugin_id).await?;
        self.enabler.add_to_enabled_plugins(&plugin_id).await
    }

    async fn save_force_enabled_flag(&self, plugin_id: &str) -> Result<(), PluginError> {
        let _settings_lock = self.enabler.plugin_registry.lock_settings(plugin_id).await;
        let storage = &self.enabler.plugin_settings_storage;
        let mut settings = storage.get(plugin_id).await?.unwrap_or_default();

        settings.force_enabled_at_api_version = Some(PLUGIN_API_VERSION.to_string());

        storage.upsert(plugin_id, &settings).await?;

        Ok(())
    }
}

#[async_trait]
impl ForceEnablePluginUseCasePort for ForceEnablePluginUseCase {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        self.execute(plugin_id).await
    }
}
