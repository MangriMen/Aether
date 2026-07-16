use std::sync::Arc;

use async_trait::async_trait;
use extism_convert::Msgpack;
use tokio::sync::Mutex;

use crate::features::{
    instance::{
        DownloadContext, InstanceError, PackInstallParams, PackManager,
        PackManagerCapabilityMetadata, PackMetadata, UpdateStatus,
    },
    plugins::{PackManagerHandlers, PluginInstance},
};

use crate::features::plugins::infra::extism::models::PluginInstanceExt;

/// Wraps a WASM plugin that declares `pack_managers` in its capabilities.
pub struct PluginPackManagerProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    metadata: PackManagerCapabilityMetadata,
    handlers: PackManagerHandlers,
}

impl PluginPackManagerProxy {
    pub fn new(
        instance: Arc<Mutex<dyn PluginInstance>>,
        metadata: PackManagerCapabilityMetadata,
        handlers: PackManagerHandlers,
    ) -> Self {
        Self {
            instance,
            metadata,
            handlers,
        }
    }
}

#[async_trait]
impl PackManager for PluginPackManagerProxy {
    fn metadata(&self) -> &PackManagerCapabilityMetadata {
        &self.metadata
    }

    async fn resolve_pack_metadata(
        &self,
        params: &PackInstallParams,
    ) -> Result<PackMetadata, InstanceError> {
        let handler = self.handlers.resolve_metadata.as_ref().ok_or_else(|| {
            InstanceError::UnsupportedOperation(
                "resolve_pack_metadata is not supported by this provider".into(),
            )
        })?;

        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        let source_dto: aether_core_plugin_api::v0::PackSourceDto = params.source.clone().into();

        let result: aether_core_plugin_api::v0::PackMetadataDto = plugin
            .call::<Msgpack<aether_core_plugin_api::v0::PackSourceDto>, Msgpack<aether_core_plugin_api::v0::PackMetadataDto>>(
                handler,
                Msgpack(source_dto),
            )
            .map(|res| res.0)
            .map_err(|err| {
                tracing::error!(
                    "Error calling pack manager resolve_pack_metadata in plugin '{}': {:?}",
                    plugin_id,
                    err
                );
                InstanceError::ContentProviderError {
                    reason: err.to_string(),
                }
            })?;

        Ok(result.into())
    }

    async fn install(
        &self,
        instance_id: &str,
        params: &PackInstallParams,
        _ctx: &DownloadContext,
    ) -> Result<(), InstanceError> {
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(&self.handlers.install) {
            return Err(InstanceError::PackManagerNotFound {
                plugin_id,
                capability_id: self.metadata.base.id.clone(),
            });
        }

        let install_params = aether_core_plugin_api::v0::PackInstallParamsDto {
            instance_id: instance_id.to_owned(),
            pack_source: params.source.clone().into(),
        };

        plugin
            .call::<Msgpack<aether_core_plugin_api::v0::PackInstallParamsDto>, ()>(
                &self.handlers.install,
                Msgpack(install_params),
            )
            .map_err(|err| {
                tracing::error!(
                    "Error calling pack manager install in plugin '{}': {:?}",
                    plugin_id,
                    err
                );
                InstanceError::ContentProviderError {
                    reason: err.to_string(),
                }
            })
    }

    async fn update(&self, instance_id: &str, _ctx: &DownloadContext) -> Result<(), InstanceError> {
        let handler = self.handlers.update.as_ref().ok_or_else(|| {
            InstanceError::UnsupportedOperation("Update is not supported by this provider".into())
        })?;

        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        plugin
            .call::<Msgpack<String>, ()>(handler, Msgpack(instance_id.to_owned()))
            .map_err(|err| {
                tracing::error!(
                    "Error calling pack manager update in plugin '{}': {:?}",
                    plugin_id,
                    err
                );
                InstanceError::ContentProviderError {
                    reason: err.to_string(),
                }
            })
    }

    async fn check_updates(&self, instance_id: &str) -> Result<UpdateStatus, InstanceError> {
        let handler = self.handlers.check_updates.as_ref().ok_or_else(|| {
            InstanceError::UnsupportedOperation(
                "Check updates is not supported by this provider".into(),
            )
        })?;

        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        plugin
            .call::<Msgpack<String>, Msgpack<UpdateStatus>>(
                handler,
                Msgpack(instance_id.to_owned()),
            )
            .map(|res| res.0)
            .map_err(|err| {
                tracing::error!(
                    "Error calling pack manager check_updates in plugin '{}': {:?}",
                    plugin_id,
                    err
                );
                InstanceError::ContentProviderError {
                    reason: err.to_string(),
                }
            })
    }
}
