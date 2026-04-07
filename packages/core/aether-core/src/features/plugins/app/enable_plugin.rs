use std::sync::Arc;

use crate::{
    features::{
        events::EventEmitter,
        plugins::{
            LoadConfigType, PluginError, PluginLoader, PluginLoaderRegistry, PluginManifest,
            PluginRegistry, PluginSettingsStorage, PluginState,
        },
        settings::SettingsStorage,
    },
    shared::UpdateAction,
};

pub struct EnablePluginUseCase<
    PSS: PluginSettingsStorage,
    SS: SettingsStorage,
    PL: PluginLoader,
    E: EventEmitter,
> {
    plugin_registry: Arc<PluginRegistry<E>>,
    plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
    plugin_settings_storage: Arc<PSS>,
    settings_storage: Arc<SS>,
}

impl<PSS: PluginSettingsStorage, SS: SettingsStorage, PL: PluginLoader, E: EventEmitter>
    EnablePluginUseCase<PSS, SS, PL, E>
{
    pub fn new(
        plugin_registry: Arc<PluginRegistry<E>>,
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

    pub async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        let plugin = self.plugin_registry.get(&plugin_id)?;
        let state = plugin.state.clone();
        let manifest = plugin.manifest.clone();
        // Drop immediate to prevent dead lock in dash map
        drop(plugin);

        self.check_is_able_to_load(&plugin_id, &state)?;

        self.plugin_registry
            .upsert_with(&plugin_id, |plugin| {
                plugin.state = PluginState::Loading;
                Ok(())
            })
            .await?;

        match self.load_plugin(&plugin_id, &manifest).await {
            Ok(_) => self.add_to_enabled_plugins(&plugin_id).await,
            Err(err) => {
                self.plugin_registry
                    .upsert_with(&plugin_id, |plugin| {
                        match &err {
                            PluginError::LoadFailed { .. } => {
                                plugin.state = PluginState::Failed(err.to_string())
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
        &self,
        plugin_id: &str,
        plugin_state: &PluginState,
    ) -> Result<(), PluginError> {
        match plugin_state {
            PluginState::NotLoaded | PluginState::Failed(_) => Ok(()),
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
