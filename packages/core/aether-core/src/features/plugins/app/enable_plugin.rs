use std::sync::Arc;

use crate::features::{
    plugins::{
        Compatibility, LoadConfigType, PLUGIN_API_VERSION, PluginError, PluginLoaderRegistry,
        PluginManifest, PluginRegistry, PluginSettingsStorage, PluginState,
    },
    settings::SettingsStorage,
};

pub struct EnablePluginUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    settings_storage: Arc<dyn SettingsStorage>,
}

impl EnablePluginUseCase {
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_loader_registry: Arc<PluginLoaderRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        settings_storage: Arc<dyn SettingsStorage>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_loader_registry,
            plugin_settings_storage,
            settings_storage,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        let (state, manifest) = self.plugin_registry.get_state_and_manifest(&plugin_id)?;

        Self::check_is_able_to_load(&plugin_id, &state)?;

        // Check if plugin was force-enabled by user (persisted in plugin settings)
        let plugin_settings = self.plugin_settings_storage.get(&plugin_id).await?;
        let is_force_enabled = plugin_settings
            .as_ref()
            .and_then(|s| s.force_enabled_at_api_version.as_deref())
            .is_some_and(|v| v == PLUGIN_API_VERSION.to_string());

        if !is_force_enabled {
            // Run compatibility check
            match manifest.api.check_compatibility(&PLUGIN_API_VERSION) {
                Compatibility::Compatible => { /* proceed */ }
                Compatibility::MajorMismatch { required, host } => {
                    let reason = format!(
                        "Plugin requires API {required}, host version is {host} — major version mismatch",
                    );
                    self.plugin_registry
                        .upsert_with(&plugin_id, |plugin| {
                            plugin.state = PluginState::Incompatible(reason.clone());
                            Ok(())
                        })
                        .await?;
                    return Err(PluginError::IncompatibleApiVersion { plugin_id, reason });
                }
                Compatibility::Incompatible(reason) => {
                    self.plugin_registry
                        .upsert_with(&plugin_id, |plugin| {
                            plugin.state = PluginState::Incompatible(reason.clone());
                            Ok(())
                        })
                        .await?;
                    return Err(PluginError::IncompatibleApiVersion { plugin_id, reason });
                }
            }
        }

        self.plugin_registry
            .upsert_with(&plugin_id, |plugin| {
                plugin.state = PluginState::Loading;
                Ok(())
            })
            .await?;

        match self.load_plugin(&plugin_id, &manifest).await {
            Ok(()) => self.add_to_enabled_plugins(&plugin_id).await,
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

    fn check_is_able_to_load(
        plugin_id: &str,
        plugin_state: &PluginState,
    ) -> Result<(), PluginError> {
        match plugin_state {
            PluginState::NotLoaded | PluginState::Failed(_) | PluginState::Incompatible(_) => {
                Ok(())
            }
            PluginState::Loading => Err(PluginError::LoadingInProgress {
                plugin_id: plugin_id.to_owned(),
            }),
            PluginState::Loaded(_) => Err(PluginError::AlreadyLoaded {
                plugin_id: plugin_id.to_owned(),
            }),
            PluginState::Unloading => Err(PluginError::UnloadingInProgress {
                plugin_id: plugin_id.to_owned(),
            }),
        }
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
        let mut settings = self.settings_storage.get().await?;
        settings.enable_plugin(plugin_id);
        self.settings_storage.upsert(settings).await?;
        Ok(())
    }
}
