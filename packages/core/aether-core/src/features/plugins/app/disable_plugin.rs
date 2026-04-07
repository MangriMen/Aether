use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    features::{
        events::EventEmitter,
        plugins::{
            LoadConfigType, PluginError, PluginInstance, PluginLoader, PluginLoaderRegistry,
            PluginManifest, PluginRegistry, PluginState,
        },
        settings::SettingsStorage,
    },
    shared::UpdateAction,
};

pub struct DisablePluginUseCase<SS: SettingsStorage, PL: PluginLoader, E: EventEmitter> {
    plugin_registry: Arc<PluginRegistry<E>>,
    plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage, PL: PluginLoader, E: EventEmitter> DisablePluginUseCase<SS, PL, E> {
    pub fn new(
        plugin_registry: Arc<PluginRegistry<E>>,
        plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
        settings_storage: Arc<SS>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_loader_registry,
            settings_storage,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        let plugin = self.plugin_registry.get(&plugin_id)?;
        let manifest = plugin.manifest.clone();
        let state = plugin.state.clone();
        // Drop immediate to prevent dead lock in dash map
        drop(plugin);

        let plugin_instance = self.check_is_able_to_unload(&plugin_id, &state)?;

        self.plugin_registry
            .upsert_with(&plugin_id, |plugin| {
                plugin.state = PluginState::Unloading;
                Ok(())
            })
            .await?;

        match self
            .unload_plugin(&plugin_id, &manifest, plugin_instance.clone())
            .await
        {
            Ok(_) => self.remove_from_enabled_plugins(&plugin_id).await,
            Err(err) => {
                self.plugin_registry
                    .upsert_with(&plugin_id, |plugin| {
                        match &err {
                            // If there is error on plugin on_unload call we are anyway drop instance
                            PluginError::FunctionCallFailed { .. } => {
                                plugin.state = PluginState::NotLoaded
                            }
                            // If there is others error - plugin still loaded
                            _ => plugin.state = PluginState::Loaded(plugin_instance),
                        }
                        Ok(())
                    })
                    .await?;

                Err(err)
            }
        }
    }

    fn check_is_able_to_unload(
        &self,
        plugin_id: &str,
        plugin_state: &PluginState,
    ) -> Result<Arc<Mutex<dyn PluginInstance>>, PluginError> {
        match plugin_state {
            PluginState::Loaded(plugin_instance) => Ok(plugin_instance.clone()),
            PluginState::Loading => Err(PluginError::LoadingInProgress {
                plugin_id: plugin_id.to_owned(),
            }),
            PluginState::NotLoaded | PluginState::Unloading | PluginState::Failed(_) => {
                Err(PluginError::AlreadyUnloaded {
                    plugin_id: plugin_id.to_owned(),
                })
            }
        }
    }

    async fn unload_plugin(
        &self,
        plugin_id: &str,
        manifest: &PluginManifest,
        plugin_instance: Arc<Mutex<dyn PluginInstance>>,
    ) -> Result<(), PluginError> {
        let load_config_type: LoadConfigType = (&manifest.load).into();
        let loader = self.plugin_loader_registry.get(&load_config_type)?;

        loader.unload(plugin_instance.clone()).await?;

        self.plugin_registry
            .upsert_with(plugin_id, |plugin| {
                plugin.state = PluginState::NotLoaded;
                Ok(())
            })
            .await?;

        Ok(())
    }

    async fn remove_from_enabled_plugins(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.settings_storage
            .upsert_with(|settings| {
                if settings.disable_plugin(plugin_id) {
                    UpdateAction::Save(())
                } else {
                    UpdateAction::NoChanges(())
                }
            })
            .await?;

        Ok(())
    }
}
