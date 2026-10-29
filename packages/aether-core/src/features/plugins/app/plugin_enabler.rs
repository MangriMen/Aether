use std::sync::Arc;

use crate::features::{
    instance::PluginId,
    plugins::{
        FolderMounts, LoadConfigType, PluginError, PluginLoaderRegistry, PluginManifest,
        PluginRegistry, PluginSettingsStorage, PluginState, PluginStateReason,
    },
    settings::SettingsStorage,
};

/// The steps enabling a plugin takes whether or not the user forces it: what differs between
/// [`EnablePluginUseCase`](super::EnablePluginUseCase) and
/// [`ForceEnablePluginUseCase`](super::ForceEnablePluginUseCase) is only the API version check.
pub(super) struct PluginEnabler {
    pub plugin_registry: Arc<PluginRegistry>,
    pub plugin_loader_registry: Arc<PluginLoaderRegistry>,
    pub plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    pub settings_storage: Arc<dyn SettingsStorage>,
}

impl PluginEnabler {
    /// What holds for every plugin, forced or not: the runtime restrictions (e.g. no absolute
    /// `allowed_paths`) and the host function declaration. A plugin failing them is marked
    /// `Incompatible`.
    pub(super) async fn check_requirements(
        &self,
        plugin_id: &PluginId,
        manifest: &PluginManifest,
    ) -> Result<(), PluginError> {
        let loader = self.plugin_loader_registry.get(&(&manifest.load).into())?;
        if let Err(err) = manifest.validate_requirements(&loader.host_functions()) {
            self.mark_incompatible(
                plugin_id,
                PluginStateReason::InvalidManifest {
                    details: err.to_string(),
                },
            )
            .await?;
            return Err(PluginError::Manifest(err));
        }

        // `api.features` is not enforced: requesting plugins load, but the gap is logged.
        if !manifest.api.features.is_empty() {
            tracing::warn!(
                %plugin_id,
                features = ?manifest.api.features,
                "Plugin requests optional API features that are not yet enforced by the host",
            );
        }

        Ok(())
    }

    pub(super) async fn mark_incompatible(
        &self,
        plugin_id: &PluginId,
        reason: PluginStateReason,
    ) -> Result<(), PluginError> {
        self.set_state(plugin_id, PluginState::Incompatible(reason))
            .await
    }

    pub(super) async fn set_state(
        &self,
        plugin_id: &PluginId,
        state: PluginState,
    ) -> Result<(), PluginError> {
        self.plugin_registry
            .upsert_with(plugin_id, |plugin| {
                plugin.state = state;
                Ok(())
            })
            .await
    }

    /// Load a plugin the caller has put in `Loading`: `Loaded` — or, when loading fails,
    /// `Failed` for a failure of the plugin itself and `NotLoaded` otherwise.
    pub(super) async fn load(
        &self,
        plugin_id: &PluginId,
        manifest: &PluginManifest,
    ) -> Result<(), PluginError> {
        let result = async {
            let load_config_type: LoadConfigType = (&manifest.load).into();
            let loader = self.plugin_loader_registry.get(&load_config_type)?;

            let plugin_settings = self.plugin_settings_storage.get(plugin_id).await?;
            let saved_folders = FolderMounts::of(
                plugin_settings
                    .as_ref()
                    .map_or(&[][..], |settings| settings.allowed_paths.as_slice()),
            );
            // The loader says which of them it mounted.
            let handle = loader.load(manifest, plugin_settings.as_ref()).await?;
            Ok::<_, PluginError>(handle.with_saved_folders(saved_folders))
        }
        .await;

        match result {
            Ok(plugin_instance) => {
                self.set_state(plugin_id, PluginState::Loaded(plugin_instance))
                    .await
            }
            Err(err) => {
                let state = match &err {
                    PluginError::LoadFailed { .. } => PluginState::Failed(load_failure(&err)),
                    _ => PluginState::NotLoaded,
                };
                self.set_state(plugin_id, state).await?;
                Err(err)
            }
        }
    }

    /// Remember the plugin as enabled, to load it on the next start.
    pub(super) async fn add_to_enabled_plugins(
        &self,
        plugin_id: &PluginId,
    ) -> Result<(), PluginError> {
        let owned = plugin_id.to_owned();
        self.settings_storage
            .update_mut(Box::new(move |mut settings| {
                let changed = settings.enable_plugin(&owned);
                (settings, changed)
            }))
            .await?;
        Ok(())
    }
}

/// The loader's own explanation when it has one: the plugin id and "failed to load" around it
/// are already said by the state the reason goes with.
pub(super) fn load_failure(err: &PluginError) -> PluginStateReason {
    let details = match err {
        PluginError::LoadFailed { reason, .. } => reason.clone(),
        other => other.to_string(),
    };
    PluginStateReason::LoadFailed { details }
}
