use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::features::{
    instance::{InstanceError, Updater, UpdaterCapabilityMetadata},
    plugins::{PluginInstance, PluginInstanceExt, PluginUpdaterCapability},
};

pub struct PluginUpdaterProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    capability: PluginUpdaterCapability,
}

impl PluginUpdaterProxy {
    pub fn new(
        instance: Arc<Mutex<dyn PluginInstance>>,
        capability: PluginUpdaterCapability,
    ) -> Self {
        Self {
            instance,
            capability,
        }
    }
}

#[async_trait]
impl Updater for PluginUpdaterProxy {
    fn metadata(&self) -> &UpdaterCapabilityMetadata {
        &self.capability
    }

    async fn update(&self, instance_id: &str) -> Result<(), InstanceError> {
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(&self.capability.handler) {
            tracing::error!(
                "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                plugin_id.clone(),
                self.capability.handler,
                self.capability.id
            );

            return Err(InstanceError::UpdateFailed {
                modpack_id: self.capability.id.clone(),
            });
        }

        plugin
            .call(&self.capability.handler, instance_id)
            .map_err(|err| {
                tracing::error!(
                    "Error updating instance by plugin '{}': {:?}",
                    plugin_id,
                    err
                );

                InstanceError::UpdateFailed {
                    modpack_id: self.capability.id.clone(),
                }
            })
    }
}
