use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::features::{
    instance::{Importer, ImporterCapabilityMetadata, InstanceError},
    plugins::{PluginImportInstance, PluginImporterCapability, PluginInstance, PluginInstanceExt},
};

pub struct PluginImporterProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    capability: PluginImporterCapability,
}

impl PluginImporterProxy {
    pub fn new(
        instance: Arc<Mutex<dyn PluginInstance>>,
        capability: PluginImporterCapability,
    ) -> Self {
        Self {
            instance,
            capability,
        }
    }
}

#[async_trait]
impl Importer for PluginImporterProxy {
    fn metadata(&self) -> &ImporterCapabilityMetadata {
        &self.capability
    }

    async fn import(&self, path: &str) -> Result<(), InstanceError> {
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(&self.capability.handler) {
            tracing::error!(
                "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                plugin_id.clone(),
                self.capability.handler,
                self.capability.id
            );

            return Err(InstanceError::ImportFailed {
                plugin_id: plugin_id.clone(),
                capability_id: self.capability.id.clone(),
            });
        }

        plugin
            .call(
                &self.capability.handler,
                PluginImportInstance {
                    importer_id: self.capability.id.clone(),
                    path: path.to_owned(),
                },
            )
            .map_err(|err| {
                tracing::error!(
                    "Error importing instance by plugin '{}': {:?}",
                    plugin_id,
                    err
                );

                InstanceError::ImportFailed {
                    plugin_id: plugin_id.clone(),
                    capability_id: self.capability.id.clone(),
                }
            })
    }
}
