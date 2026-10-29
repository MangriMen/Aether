use async_trait::async_trait;

use crate::features::{
    instance::{InstanceError, InstanceId, Updater, UpdaterCapabilityMetadata},
    plugins::{PluginError, PluginHandle, PluginUpdaterCapability},
};

use crate::features::plugins::infra::extism::models::PluginInstanceExt;

pub struct PluginUpdaterProxy {
    instance: PluginHandle,
    capability: PluginUpdaterCapability,
}

impl PluginUpdaterProxy {
    pub fn new(instance: PluginHandle, capability: PluginUpdaterCapability) -> Self {
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

    async fn update(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let plugin_id = self.instance.id();
        let failed = || InstanceError::UpdateFailed {
            modpack_id: self.capability.id.to_string(),
        };

        let handler = self.capability.handler.clone();
        let instance_id = instance_id.clone();

        let result = self
            .instance
            .run(&self.capability.handler, move |plugin| {
                plugin
                    .supports(&handler)
                    .then(|| plugin.call::<_, ()>(&handler, instance_id.as_str()))
            })
            .await
            .and_then(Option::transpose);

        match result {
            Ok(Some(())) => Ok(()),
            Err(PluginError::Stopped { plugin_id }) => Err(InstanceError::PluginNotRunning {
                plugin_id: plugin_id.into(),
            }),
            Err(err) => {
                tracing::error!("Error updating instance by plugin '{plugin_id}': {err:?}");
                Err(failed())
            }
            Ok(None) => {
                tracing::error!(
                    "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                    plugin_id,
                    self.capability.handler,
                    self.capability.id
                );
                Err(failed())
            }
        }
    }
}
