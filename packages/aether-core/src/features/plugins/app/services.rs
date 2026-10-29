use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::{
    multiple::RefMulti as DashMapRefMulti,
    one::{Ref as DashMapRef, RefMut as DashMapRefMut},
};
use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::features::{
    events::{EventEmitterExt, PluginEvent, SharedEventEmitter},
    plugins::domain::LoadConfigType,
    plugins::{Plugin, PluginCapabilities, PluginError, PluginHandle, PluginManifest, PluginState},
};

use super::ports::PluginLoader;

pub struct PluginRegistry {
    plugins: DashMap<String, Plugin>,
    settings_locks: DashMap<String, Arc<Mutex<()>>>,
    event_emitter: SharedEventEmitter,
}

impl PluginRegistry {
    pub fn new(event_emitter: SharedEventEmitter) -> Self {
        Self {
            plugins: DashMap::default(),
            settings_locks: DashMap::default(),
            event_emitter,
        }
    }

    /// Held while a plugin's settings are read, changed and saved: every change rewrites the
    /// whole settings, so two at once would save over each other. Not to be held across a
    /// restart, which takes seconds and no settings change needs to wait for.
    pub async fn lock_settings(&self, plugin_id: &str) -> OwnedMutexGuard<()> {
        let lock = Arc::clone(&self.settings_locks.entry(plugin_id.to_owned()).or_default());
        lock.lock_owned().await
    }

    pub fn insert(&self, plugin_id: String, plugin: Plugin) {
        self.plugins.insert(plugin_id, plugin);
    }

    pub fn list(&self) -> impl Iterator<Item = DashMapRefMulti<'_, String, Plugin>> {
        self.plugins.iter()
    }

    pub fn get(&self, plugin_id: &str) -> Result<DashMapRef<'_, String, Plugin>, PluginError> {
        self.plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound {
                plugin_id: plugin_id.to_owned(),
            })
    }

    pub fn remove(&self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }

    pub fn get_ids(&self) -> HashSet<String> {
        self.plugins
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn get_enabled_ids(&self) -> HashSet<String> {
        self.plugins
            .iter()
            .filter_map(|entry| match &entry.state {
                PluginState::Loaded(_) => Some(entry.key().clone()),
                _ => None,
            })
            .collect()
    }

    pub fn list_manifests(&self) -> Result<Vec<PluginManifest>, PluginError> {
        Ok(self.list().map(|plugin| plugin.manifest.clone()).collect())
    }

    pub fn get_capabilities(
        &self,
        plugin_id: &str,
    ) -> Result<Option<PluginCapabilities>, PluginError> {
        Ok(self.get(plugin_id)?.capabilities.clone())
    }

    pub fn get_manifest(&self, plugin_id: &str) -> Result<PluginManifest, PluginError> {
        Ok(self.get(plugin_id)?.manifest.clone())
    }

    /// Clone state and manifest then drop the `DashMap` lock immediately.
    /// Prevents deadlocks when `upsert_with()` is called later.
    pub fn get_state_and_manifest(
        &self,
        plugin_id: &str,
    ) -> Result<(PluginState, PluginManifest), PluginError> {
        let plugin = self.get(plugin_id)?;
        let state = plugin.state.clone();
        let manifest = plugin.manifest.clone();
        // Lock released on drop(plugin)
        Ok((state, manifest))
    }

    pub async fn upsert_with<F>(&self, plugin_id: &str, update_fn: F) -> Result<(), PluginError>
    where
        F: FnOnce(&mut Plugin) -> Result<(), PluginError> + Send,
    {
        // The entry guard is released before the `await`: a handler that touches the same entry
        // would otherwise block on the guard we hold across the suspension point.
        {
            let mut plugin = self.get_mut(plugin_id)?;
            update_fn(&mut plugin)?;
        }

        self.event_emitter
            .emit_safe(PluginEvent::Edit {
                plugin_id: plugin_id.to_owned(),
            })
            .await;

        Ok(())
    }

    /// Takes the plugin to `Loading` in the same step that checks it may load, so that of two
    /// enables only one builds an instance; the other gets [`PluginError::LoadingInProgress`].
    /// Returns the state it was in, for a caller that gives up before loading, and its manifest.
    pub async fn begin_loading(
        &self,
        plugin_id: &str,
    ) -> Result<(PluginState, PluginManifest), PluginError> {
        let mut taken = None;
        self.upsert_with(plugin_id, |plugin| {
            let plugin_id = plugin_id.to_owned();
            match &plugin.state {
                PluginState::NotLoaded | PluginState::Failed(_) | PluginState::Incompatible(_) => {}
                PluginState::Loading => return Err(PluginError::LoadingInProgress { plugin_id }),
                PluginState::Loaded(_) => return Err(PluginError::AlreadyLoaded { plugin_id }),
                PluginState::Unloading | PluginState::Removing => {
                    return Err(PluginError::UnloadingInProgress { plugin_id });
                }
            }
            let previous = std::mem::replace(&mut plugin.state, PluginState::Loading);
            taken = Some((previous, plugin.manifest.clone()));
            Ok(())
        })
        .await?;

        // The closure assigns `taken` before it returns `Ok`; every other path returned an error.
        Ok(taken.expect("the state is taken whenever it is changed"))
    }

    /// Puts back the state [`Self::begin_loading`] took the plugin from, unless something was
    /// decided about it since (it was found incompatible).
    pub async fn abandon_loading(
        &self,
        plugin_id: &str,
        previous: PluginState,
    ) -> Result<(), PluginError> {
        self.upsert_with(plugin_id, |plugin| {
            if matches!(plugin.state, PluginState::Loading) {
                plugin.state = previous;
            }
            Ok(())
        })
        .await
    }

    /// Takes the plugin from `Loaded` to `Unloading` in one step, so that two restarts, or a
    /// restart and a disable, cannot both take the same instance. A busy plugin is refused
    /// without passing through `Unloading`.
    pub async fn begin_unloading(&self, plugin_id: &str) -> Result<PluginHandle, PluginError> {
        let mut taken = None;
        self.upsert_with(plugin_id, |plugin| {
            let plugin_id = plugin_id.to_owned();
            let handle = match &plugin.state {
                PluginState::Loaded(handle) => handle,
                PluginState::Loading => {
                    return Err(PluginError::LoadingInProgress { plugin_id });
                }
                PluginState::Unloading | PluginState::Removing => {
                    return Err(PluginError::UnloadingInProgress { plugin_id });
                }
                PluginState::NotLoaded | PluginState::Failed(_) | PluginState::Incompatible(_) => {
                    return Err(PluginError::AlreadyUnloaded { plugin_id });
                }
            };
            // The retirement checks this again for good.
            if handle.is_busy() {
                return Err(PluginError::Busy { plugin_id });
            }
            taken = Some(handle.clone());
            plugin.state = PluginState::Unloading;
            Ok(())
        })
        .await?;

        // The closure assigns `taken` before it returns `Ok`; every other path returned an error.
        Ok(taken.expect("the handle is taken whenever the state is changed"))
    }

    /// Takes a plugin that is not running to `Removing` in the step that checks nothing is
    /// loading it, so that of a remove and an enable only one goes on. Returns the state it
    /// was in, to put back should the delete fail. A plugin that is running, or in the middle of
    /// a transition, is refused with the matching error and its files are not to be touched.
    pub async fn begin_removing(&self, plugin_id: &str) -> Result<PluginState, PluginError> {
        let mut taken = None;
        self.upsert_with(plugin_id, |plugin| {
            let plugin_id = plugin_id.to_owned();
            match &plugin.state {
                PluginState::NotLoaded | PluginState::Failed(_) | PluginState::Incompatible(_) => {}
                PluginState::Loading => return Err(PluginError::LoadingInProgress { plugin_id }),
                PluginState::Loaded(_) => return Err(PluginError::AlreadyLoaded { plugin_id }),
                PluginState::Unloading | PluginState::Removing => {
                    return Err(PluginError::UnloadingInProgress { plugin_id });
                }
            }
            taken = Some(std::mem::replace(&mut plugin.state, PluginState::Removing));
            Ok(())
        })
        .await?;

        Ok(taken.expect("the state is taken whenever it is changed"))
    }

    /// Puts back the state [`Self::begin_removing`] took the plugin from.
    pub async fn abandon_removing(
        &self,
        plugin_id: &str,
        previous: PluginState,
    ) -> Result<(), PluginError> {
        self.upsert_with(plugin_id, |plugin| {
            if matches!(plugin.state, PluginState::Removing) {
                plugin.state = previous;
            }
            Ok(())
        })
        .await
    }

    /// The plugins whose files are being deleted.
    pub fn get_removing_ids(&self) -> HashSet<String> {
        self.plugins
            .iter()
            .filter(|entry| matches!(entry.state, PluginState::Removing))
            .map(|entry| entry.key().clone())
            .collect()
    }

    fn get_mut(&self, plugin_id: &str) -> Result<DashMapRefMut<'_, String, Plugin>, PluginError> {
        self.plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound {
                plugin_id: plugin_id.to_owned(),
            })
    }
}

#[derive(Default)]
pub struct PluginLoaderRegistry {
    loaders: HashMap<LoadConfigType, Arc<dyn PluginLoader>>,
}

impl PluginLoaderRegistry {
    pub fn new(loaders: HashMap<LoadConfigType, Arc<dyn PluginLoader>>) -> Self {
        Self { loaders }
    }

    pub fn get(
        &self,
        load_config_type: &LoadConfigType,
    ) -> Result<&Arc<dyn PluginLoader>, PluginError> {
        self.loaders
            .get(load_config_type)
            .ok_or(PluginError::LoaderNotFound {
                config_type: *load_config_type,
            })
    }

    pub fn register(&mut self, load_config_type: LoadConfigType, provider: Arc<dyn PluginLoader>) {
        self.loaders.insert(load_config_type, provider);
    }

    pub fn unregister(&mut self, load_config_type: &LoadConfigType) {
        self.loaders.remove(load_config_type);
    }
}
