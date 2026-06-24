use std::sync::Arc;

use crate::{
    features::{
        plugins::{
            LoadConfigType, PLUGIN_API_VERSION, PluginError, PluginLoader, PluginLoaderRegistry,
            PluginManifest, PluginRegistry, PluginSettingsStorage, PluginState,
        },
        settings::SettingsStorage,
    },
    shared::json_store::domain::UpdateAction,
};

/// Force-enables a plugin by skipping the API version compatibility check
/// and persisting the user's choice in the plugin settings.
///
/// On next restart the plugin will auto-load if the API version hasn't
/// changed. If the launcher is updated, the flag is ignored and the user
/// must re-confirm.
pub struct ForceEnablePluginUseCase<
    PSS: PluginSettingsStorage,
    SS: SettingsStorage,
    PL: PluginLoader,
> {
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
    plugin_settings_storage: Arc<PSS>,
    settings_storage: Arc<SS>,
}

impl<PSS: PluginSettingsStorage, SS: SettingsStorage, PL: PluginLoader>
    ForceEnablePluginUseCase<PSS, SS, PL>
{
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
        plugin_settings_storage: Arc<PSS>,
        settings_storage: Arc<SS>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_loader_registry,
            plugin_settings_storage,
            settings_storage,
        }
    }

    /// Bypasses the API version check and loads the plugin directly.
    /// First resets any `Incompatible` state and saves the force-enable
    /// flag in plugin settings with the current API version.
    pub async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        let (state, manifest) = self.plugin_registry.get_state_and_manifest(&plugin_id)?;

        // Reset incompatible / failed state so load can proceed
        if matches!(state, PluginState::Incompatible(_) | PluginState::Failed(_)) {
            self.plugin_registry
                .upsert_with(&plugin_id, |plugin| {
                    plugin.state = PluginState::NotLoaded;
                    Ok(())
                })
                .await?;
        }

        self.plugin_registry
            .upsert_with(&plugin_id, |plugin| {
                plugin.state = PluginState::Loading;
                Ok(())
            })
            .await?;

        match self.load_plugin(&plugin_id, &manifest).await {
            Ok(()) => {
                // Persist the force-enable flag in plugin settings
                self.save_force_enabled_flag(&plugin_id).await?;
                self.add_to_enabled_plugins(&plugin_id).await
            }
            Err(err) => {
                self.plugin_registry
                    .upsert_with(&plugin_id, |plugin| {
                        match &err {
                            PluginError::LoadFailed { .. } => {
                                plugin.state = PluginState::Failed(err.to_string());
                            }
                            _ => plugin.state = PluginState::NotLoaded,
                        }
                        Ok(())
                    })
                    .await?;
                Err(err)
            }
        }
    }

    async fn save_force_enabled_flag(&self, plugin_id: &str) -> Result<(), PluginError> {
        let mut settings = self
            .plugin_settings_storage
            .get(plugin_id)
            .await?
            .unwrap_or_default();

        settings.force_enabled_at_api_version = Some(PLUGIN_API_VERSION.to_string());

        self.plugin_settings_storage
            .upsert(plugin_id, &settings)
            .await?;

        Ok(())
    }

    async fn load_plugin(
        &self,
        plugin_id: &str,
        manifest: &PluginManifest,
    ) -> Result<(), PluginError> {
        let load_config_type: LoadConfigType = (&manifest.load).into();
        let loader = self.plugin_loader_registry.get(&load_config_type)?;

        let plugin_settings = self.plugin_settings_storage.get(plugin_id).await?;
        let plugin_instance = loader.load(manifest, plugin_settings.as_ref()).await?;

        self.plugin_registry
            .upsert_with(plugin_id, |plugin| {
                plugin.state = PluginState::Loaded(plugin_instance);
                Ok(())
            })
            .await?;

        Ok(())
    }

    async fn add_to_enabled_plugins(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.settings_storage
            .upsert_with(|settings| {
                if settings.enable_plugin(plugin_id) {
                    UpdateAction::Save(())
                } else {
                    UpdateAction::NoChanges(())
                }
            })
            .await?;

        Ok(())
    }
}
