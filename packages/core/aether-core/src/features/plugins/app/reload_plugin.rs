use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    plugins::{
        PluginError, PluginHandle, PluginLoaderRegistry, PluginManifest, PluginRegistry,
        PluginSettingsStorage, PluginState,
    },
    settings::SettingsStorage,
};

use super::{
    plugin_enabler::{PluginEnabler, load_failure},
    ports::ReloadPluginUseCasePort,
};

/// Restarts a loaded plugin with a new instance built from its current settings, which is what
/// makes changed folders take effect. Wasm memory is lost with the old instance.
///
/// The plugin stays enabled whatever happens; a failed load leaves it `Failed`. The state skips
/// `NotLoaded`, which would take its capabilities out of the registries during the restart.
pub(crate) struct ReloadPluginUseCase {
    enabler: PluginEnabler,
}

impl ReloadPluginUseCase {
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

    /// Refused with [`PluginError::Busy`] while any call into the plugin is under way or waiting
    /// for its turn — the plugin then goes on as it was. Never waits for a call to end.
    pub(crate) async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        let manifest = self.enabler.plugin_registry.get_manifest(&plugin_id)?;
        let handle = self
            .enabler
            .plugin_registry
            .begin_unloading(&plugin_id)
            .await?;

        if let Err(err) = self.unload(&manifest, handle.clone()).await {
            if !handle.is_retired() {
                self.enabler
                    .set_state(&plugin_id, PluginState::Loaded(handle))
                    .await?;
                return Err(err);
            }
            tracing::debug!("Plugin {plugin_id} failed to unload cleanly on restart: {err}");
        }

        // Nothing else takes a plugin that is `Unloading`, so the restart still holds it here.
        self.enabler
            .set_state(&plugin_id, PluginState::Loading)
            .await?;
        if let Err(err) = self.enabler.load(&plugin_id, &manifest).await {
            // Enabled but not running is a failure to show, whatever made the load fail.
            self.enabler
                .set_state(&plugin_id, PluginState::Failed(load_failure(&err)))
                .await?;
            return Err(err);
        }

        Ok(())
    }

    async fn unload(
        &self,
        manifest: &PluginManifest,
        handle: PluginHandle,
    ) -> Result<(), PluginError> {
        let loader = self
            .enabler
            .plugin_loader_registry
            .get(&(&manifest.load).into())?;
        loader.unload(handle).await
    }
}

#[async_trait]
impl ReloadPluginUseCasePort for ReloadPluginUseCase {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        self.execute(plugin_id).await
    }
}
