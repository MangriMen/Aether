//! What the tests of plugins share.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::Notify;

use crate::features::{
    instance::PluginId,
    plugins::{PluginError, PluginSettings, PluginSettingsStorage},
};

/// Hands out what it holds, then takes its time before returning it: two changes that were not
/// made one after the other both read the settings before either saves.
#[derive(Default)]
pub(crate) struct SlowPluginSettings<S> {
    pub(crate) inner: Arc<S>,
    /// Notified when a read has started, so a test can start its second change in the middle of it.
    pub(crate) read_started: Notify,
}

impl<S> SlowPluginSettings<S> {
    pub(crate) fn new(inner: Arc<S>) -> Self {
        Self {
            inner,
            read_started: Notify::new(),
        }
    }
}

#[async_trait]
impl<S: PluginSettingsStorage> PluginSettingsStorage for SlowPluginSettings<S> {
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSettings>, PluginError> {
        self.read_started.notify_one();
        let settings = self.inner.get(plugin_id).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        settings
    }

    async fn upsert(
        &self,
        plugin_id: &PluginId,
        settings: &PluginSettings,
    ) -> Result<(), PluginError> {
        self.inner.upsert(plugin_id, settings).await
    }
}
