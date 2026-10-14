use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    plugins::{
        Compatibility, PLUGIN_API_VERSION, PluginError, PluginLoaderRegistry, PluginManifest,
        PluginRegistry, PluginSettingsStorage, PluginStateReason,
    },
    settings::SettingsStorage,
};

use super::{plugin_enabler::PluginEnabler, ports::EnablePluginUseCasePort};

pub(crate) struct EnablePluginUseCase {
    enabler: PluginEnabler,
}

impl EnablePluginUseCase {
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

    pub(crate) async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        let (previous, manifest) = self
            .enabler
            .plugin_registry
            .begin_loading(&plugin_id)
            .await?;

        if let Err(err) = self.check_may_load(&plugin_id, &manifest).await {
            self.enabler
                .plugin_registry
                .abandon_loading(&plugin_id, previous)
                .await?;
            return Err(err);
        }

        self.enabler.load(&plugin_id, &manifest).await?;
        self.enabler.add_to_enabled_plugins(&plugin_id).await
    }

    /// The manifest's requirements and the API version, unless the user forced this version.
    async fn check_may_load(
        &self,
        plugin_id: &str,
        manifest: &PluginManifest,
    ) -> Result<(), PluginError> {
        // Must hold unconditionally, even for plugins the user force-enabled.
        self.enabler.check_requirements(plugin_id, manifest).await?;

        // Check if plugin was force-enabled by user (persisted in plugin settings)
        let plugin_settings = self.enabler.plugin_settings_storage.get(plugin_id).await?;
        let is_force_enabled = plugin_settings
            .as_ref()
            .and_then(|s| s.force_enabled_at_api_version.as_deref())
            .is_some_and(|v| v == PLUGIN_API_VERSION.to_string());

        if !is_force_enabled {
            // Run compatibility check
            let reason = match manifest.api.check_compatibility(&PLUGIN_API_VERSION) {
                Compatibility::Compatible => None,
                Compatibility::MajorMismatch { required, host }
                | Compatibility::Incompatible { required, host } => {
                    Some(PluginStateReason::IncompatibleApiVersion { required, host })
                }
            };

            if let Some(reason) = reason {
                let text = reason.to_string();
                self.enabler.mark_incompatible(plugin_id, reason).await?;
                return Err(PluginError::IncompatibleApiVersion {
                    plugin_id: plugin_id.to_owned(),
                    reason: text,
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EnablePluginUseCasePort for EnablePluginUseCase {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        self.execute(plugin_id).await
    }
}
