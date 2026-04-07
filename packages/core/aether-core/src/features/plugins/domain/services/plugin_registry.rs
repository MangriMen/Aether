use std::{collections::HashSet, sync::Arc};

use dashmap::DashMap;

use crate::features::{
    events::{EventEmitter, EventEmitterExt, PluginEventType},
    plugins::{Plugin, PluginCapabilities, PluginError, PluginManifest, PluginState},
};

use dashmap::mapref::{
    multiple::RefMulti as DashMapRefMulti,
    one::{Ref as DashMapRef, RefMut as DashMapRefMut},
};

#[derive(Default)]
pub struct PluginRegistry<E: EventEmitter> {
    plugins: DashMap<String, Plugin>,
    event_emitter: Arc<E>,
}

impl<E: EventEmitter> PluginRegistry<E> {
    pub fn new(event_emitter: Arc<E>) -> Self {
        Self {
            plugins: DashMap::default(),
            event_emitter,
        }
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

    pub async fn upsert_with<F>(&self, plugin_id: &str, update_fn: F) -> Result<(), PluginError>
    where
        F: FnOnce(&mut Plugin) -> Result<(), PluginError> + Send,
    {
        let mut plugin = self.get_mut(plugin_id)?;
        update_fn(&mut plugin)?;
        self.event_emitter
            .emit_plugin_safe(PluginEventType::Edit {
                plugin_id: plugin_id.to_owned(),
            })
            .await;

        Ok(())
    }

    fn get_mut(&self, plugin_id: &str) -> Result<DashMapRefMut<'_, String, Plugin>, PluginError> {
        self.plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound {
                plugin_id: plugin_id.to_owned(),
            })
    }
}
