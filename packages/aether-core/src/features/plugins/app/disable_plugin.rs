use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    instance::PluginId,
    plugins::{
        LoadConfigType, PluginDisableService, PluginError, PluginHandle, PluginLoaderRegistry,
        PluginManifest, PluginRegistry, PluginState,
    },
    settings::SettingsStorage,
};

pub(crate) struct DisablePluginUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry>,
    settings_storage: Arc<dyn SettingsStorage>,
}

impl DisablePluginUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_loader_registry: Arc<PluginLoaderRegistry>,
        settings_storage: Arc<dyn SettingsStorage>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_loader_registry,
            settings_storage,
        }
    }

    /// A plugin being enabled or restarted is refused with the matching in-progress error rather
    /// than taken from under it: a restart finishing after the disable would leave it running.
    pub(crate) async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError> {
        self.unload(plugin_id.clone()).await?;
        self.remove_from_enabled_plugins(&plugin_id).await
    }

    /// [`Self::execute`] without the plugin ceasing to be enabled.
    pub(crate) async fn unload(&self, plugin_id: PluginId) -> Result<(), PluginError> {
        let manifest = self.plugin_registry.get_manifest(&plugin_id)?;
        let plugin_instance = self.plugin_registry.begin_unloading(&plugin_id).await?;

        match self
            .unload_plugin(&plugin_id, &manifest, plugin_instance.clone())
            .await
        {
            Ok(()) => Ok(()),
            Err(err) => {
                self.plugin_registry
                    .upsert_with(&plugin_id, |plugin| {
                        // A busy plugin, refused before its instance was touched, stays loaded.
                        plugin.state = if plugin_instance.is_retired() {
                            PluginState::NotLoaded
                        } else {
                            PluginState::Loaded(plugin_instance)
                        };
                        Ok(())
                    })
                    .await?;

                Err(err)
            }
        }
    }

    async fn unload_plugin(
        &self,
        plugin_id: &PluginId,
        manifest: &PluginManifest,
        plugin_instance: PluginHandle,
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

    async fn remove_from_enabled_plugins(&self, plugin_id: &PluginId) -> Result<(), PluginError> {
        let owned = plugin_id.to_owned();
        self.settings_storage
            .update_mut(Box::new(move |mut settings| {
                let changed = settings.disable_plugin(&owned);
                (settings, changed)
            }))
            .await?;
        Ok(())
    }
}

#[async_trait]
impl PluginDisableService for DisablePluginUseCase {
    async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError> {
        self.execute(plugin_id).await
    }

    async fn unload(&self, plugin_id: PluginId) -> Result<(), PluginError> {
        self.unload(plugin_id).await
    }
}
