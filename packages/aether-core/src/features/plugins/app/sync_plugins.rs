use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::features::{
    events::{EventEmitterExt, PluginEvent, SharedEventEmitter},
    instance::PluginId,
    plugins::{
        Plugin, PluginDisableService, PluginError, PluginRegistry, PluginStorage, PluginSyncService,
    },
    settings::SettingsStorage,
};

use super::ports::EnablePluginUseCasePort;

/// Brings the registry in line with the plugins on disk. A plugin whose files changed is
/// ended and put back as it now is; if it was enabled it is enabled again, so an update takes
/// effect without the user's choice being lost.
pub(crate) struct SyncPluginsUseCase {
    plugin_storage: Arc<dyn PluginStorage>,
    plugin_registry: Arc<PluginRegistry>,
    disable_plugin_service: Arc<dyn PluginDisableService>,
    enable_plugin_service: Arc<dyn EnablePluginUseCasePort>,
    settings_storage: Arc<dyn SettingsStorage>,
    event_emitter: SharedEventEmitter,
}

impl SyncPluginsUseCase {
    pub(crate) fn new(
        plugin_storage: Arc<dyn PluginStorage>,
        plugin_registry: Arc<PluginRegistry>,
        disable_plugin_service: Arc<dyn PluginDisableService>,
        enable_plugin_service: Arc<dyn EnablePluginUseCasePort>,
        settings_storage: Arc<dyn SettingsStorage>,
        event_emitter: SharedEventEmitter,
    ) -> Self {
        Self {
            plugin_storage,
            plugin_registry,
            disable_plugin_service,
            enable_plugin_service,
            settings_storage,
            event_emitter,
        }
    }

    async fn sync_plugins(
        &self,
        mut found_plugins: HashMap<PluginId, Plugin>,
    ) -> Result<(), PluginError> {
        // A plugin being removed belongs to the remove: its directory is half deleted, and
        // adding or ending it here would undo the guard that keeps it from loading.
        let removing_plugins = self.plugin_registry.get_removing_ids();
        found_plugins.retain(|id, _| !removing_plugins.contains(id));
        let existing_plugins = &self.plugin_registry.get_ids() - &removing_plugins;
        let enabled_plugins = self.settings_storage.get().await?.enabled_plugins().clone();

        let new_plugins = found_plugins.keys().cloned().collect::<HashSet<_>>();

        let changed_plugins =
            self.find_changed_plugins(&existing_plugins, &new_plugins, &found_plugins);
        let plugins_to_add =
            Self::determine_plugins_to_add(&existing_plugins, &new_plugins, &changed_plugins);

        self.remove_non_existing_plugins(&existing_plugins, &new_plugins)
            .await?;
        self.remove_changed_plugins(&changed_plugins).await?;
        self.add_new_plugins(&plugins_to_add, &found_plugins);
        self.enable_changed_plugins(&changed_plugins, &enabled_plugins)
            .await;

        Ok(())
    }

    /// A failure leaves the plugin enabled but not running, as at start-up; the state tells why.
    async fn enable_changed_plugins(
        &self,
        changed_plugins: &HashSet<PluginId>,
        enabled_plugins: &HashSet<String>,
    ) {
        let enabled = changed_plugins
            .iter()
            .filter(|id| enabled_plugins.contains(id.as_str()));
        for plugin_id in enabled {
            if let Err(err) = self.enable_plugin_service.execute(plugin_id.clone()).await {
                log::warn!("Changed plugin {plugin_id} could not be enabled again: {err}");
            }
        }
    }

    fn find_changed_plugins(
        &self,
        existing_plugins: &HashSet<PluginId>,
        new_plugins: &HashSet<PluginId>,
        found_plugins: &HashMap<PluginId, Plugin>,
    ) -> HashSet<PluginId> {
        existing_plugins
            .intersection(new_plugins)
            .filter(|&plugin| {
                let old = self.plugin_registry.get(plugin);
                let new = found_plugins.get(plugin);

                match (old, new) {
                    (Ok(old), Some(new)) => {
                        old.manifest != new.manifest
                            || old.hash != new.hash
                            || old.verified != new.verified
                    }
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    fn determine_plugins_to_add(
        existing_plugins: &HashSet<PluginId>,
        new_plugins: &HashSet<PluginId>,
        changed_plugins: &HashSet<PluginId>,
    ) -> HashSet<PluginId> {
        new_plugins
            .difference(existing_plugins)
            .chain(changed_plugins.iter())
            .cloned()
            .collect()
    }

    async fn remove_non_existing_plugins(
        &self,
        existing_plugins: &HashSet<PluginId>,
        new_plugins: &HashSet<PluginId>,
    ) -> Result<(), PluginError> {
        for plugin_id in existing_plugins.difference(new_plugins) {
            log::debug!("Removing plugin {plugin_id}");
            self.remove_plugin(plugin_id, false).await?;
        }
        Ok(())
    }

    async fn remove_changed_plugins(
        &self,
        changed_plugins: &HashSet<PluginId>,
    ) -> Result<(), PluginError> {
        for plugin_id in changed_plugins {
            log::debug!("Removing changed plugin {plugin_id}");
            self.remove_plugin(plugin_id, true).await?;
        }
        Ok(())
    }

    fn add_new_plugins(
        &self,
        plugins_to_add: &HashSet<PluginId>,
        found_plugins: &HashMap<PluginId, Plugin>,
    ) {
        for plugin_id in plugins_to_add {
            if let Some(plugin_state) = found_plugins.get(plugin_id) {
                log::debug!("Adding plugin {plugin_id}");
                self.plugin_registry
                    .insert(plugin_id.clone(), plugin_state.clone());
            }
        }
    }

    /// `keep_enabled` for a plugin that is put back at once: it must not drop out of the
    /// enabled plugins on the way.
    async fn remove_plugin(
        &self,
        plugin_id: &PluginId,
        keep_enabled: bool,
    ) -> Result<(), PluginError> {
        let disable_result = if keep_enabled {
            self.disable_plugin_service.unload(plugin_id.clone()).await
        } else {
            self.disable_plugin_service.execute(plugin_id.clone()).await
        };

        if let Err(err) = disable_result {
            match err {
                PluginError::NotFound { .. } | PluginError::AlreadyUnloaded { .. } => {
                    log::debug!("Plugin {plugin_id} was already disabled");
                }
                _ => return Err(err),
            }
        }

        self.plugin_registry.remove(plugin_id);
        Ok(())
    }
}

#[async_trait::async_trait]
impl PluginSyncService for SyncPluginsUseCase {
    async fn execute(&self) -> Result<(), PluginError> {
        let found_plugins = self.plugin_storage.list().await?;
        self.sync_plugins(found_plugins).await?;
        self.event_emitter.emit_safe(PluginEvent::Sync).await;
        Ok(())
    }
}
