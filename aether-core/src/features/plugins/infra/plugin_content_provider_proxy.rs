use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use extism_convert::Msgpack;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;

use crate::features::{
    instance::{
        app::{ContentCompatibilityCheckParams, ContentCompatibilityResult},
        AtomicInstallParams, ContentFile, ContentProvider, ContentProviderCapabilityMetadata,
        ContentSearchParams, ContentSearchResult, Instance, InstanceError, ModpackInstallParams,
    },
    plugins::{
        PluginCheckCompatibilityParams, PluginContentProviderCapability, PluginInstance,
        PluginInstanceExt,
    },
};

pub struct PluginContentProviderProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    capability: PluginContentProviderCapability,
}

impl PluginContentProviderProxy {
    pub fn new(
        instance: Arc<Mutex<dyn PluginInstance>>,
        capability: PluginContentProviderCapability,
    ) -> Self {
        Self {
            instance,
            capability,
        }
    }

    async fn call_plugin<I, O>(&self, handler_name: &str, input: I) -> Result<O, InstanceError>
    where
        I: Serialize,
        O: DeserializeOwned,
    {
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(handler_name) {
            tracing::error!(
                "Plugin '{}' missing handler '{}' for capability '{}'",
                plugin_id,
                handler_name,
                self.capability.id
            );
            return Err(InstanceError::ContentProviderNotFound {
                plugin_id,
                capability_id: self.capability.id.clone(),
            });
        }

        plugin
            .call::<Msgpack<I>, Msgpack<O>>(handler_name, Msgpack(input))
            .map(|res| res.0)
            .map_err(|err| {
                tracing::error!(
                    "Error calling plugin '{}' ({}): {:?}",
                    plugin_id,
                    handler_name,
                    err
                );
                InstanceError::ContentProviderError {
                    reason: err.to_string(),
                }
            })
    }
}

#[async_trait]
impl ContentProvider for PluginContentProviderProxy {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.capability
    }

    async fn search(
        &self,
        search_content: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        self.call_plugin(&self.capability.search_handler, search_content)
            .await
    }

    async fn install_atomic(
        &self,
        install_params: &AtomicInstallParams,
    ) -> Result<ContentFile, InstanceError> {
        self.call_plugin(&self.capability.install_atomic_handler, install_params)
            .await
    }

    async fn install_modpack(
        &self,
        install_params: &ModpackInstallParams,
    ) -> Result<(String, Vec<ContentFile>), InstanceError> {
        self.call_plugin(&self.capability.install_modpack_handler, install_params)
            .await
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        self.call_plugin(
            &self.capability.check_compatibility_handler,
            PluginCheckCompatibilityParams {
                instances: instances.to_vec(),
                check_params: check_params.clone(),
            },
        )
        .await
    }
}
