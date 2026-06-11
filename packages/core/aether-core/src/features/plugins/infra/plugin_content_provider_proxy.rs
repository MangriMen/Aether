use std::{collections::HashMap, sync::Arc};

use aether_core_plugin_api::v0::{
    AtomicInstallParamsDto, ContentSearchParamsDto, ContentSearchResultDto, ContentVersionDto,
    DownloadedContentDto, ModpackInstallParamsDto, PluginCheckCompatibilityParamsDto,
};
use async_trait::async_trait;
use extism_convert::Msgpack;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::Mutex;

use crate::features::{
    instance::{
        AtomicInstallParams, ContentCompatibilityCheckParams, ContentCompatibilityResult,
        ContentFile, ContentItem, ContentProvider, ContentProviderCapabilityMetadata,
        ContentSearchParams, ContentSearchResult, ContentVersion, DownloadedContent, Instance,
        InstanceError, ModpackInstallParams,
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

    async fn call_optional<I, O>(
        &self,
        handler: Option<&String>,
        input: I,
    ) -> Result<O, InstanceError>
    where
        I: Serialize,
        O: DeserializeOwned,
    {
        if let Some(handler_name) = handler {
            self.call_plugin(handler_name, input).await
        } else {
            let plugin = self.instance.lock().await;
            let plugin_id = plugin.get_id();

            tracing::warn!(
                "Capability '{}' in plugin '{}' does not support this operation (handler is None)",
                self.capability.metadata.id,
                plugin_id
            );

            Err(InstanceError::ContentProviderNotFound {
                plugin_id,
                capability_id: self.capability.metadata.id.clone(),
            })
        }
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
        let dto: ContentSearchParamsDto = search_content.into();
        let result_dto: ContentSearchResultDto = self
            .call_plugin(&self.capability.handlers.search, dto)
            .await?;
        // Convert back from DTO — build domain result manually
        Ok(ContentSearchResult {
            page: result_dto.page,
            page_size: result_dto.page_size,
            page_count: result_dto.page_count,
            provider_id: result_dto.provider_id.into(),
            items: result_dto
                .items
                .into_iter()
                .map(|item| ContentItem {
                    id: item.id,
                    slug: item.slug,
                    name: item.name,
                    description: item.description,
                    long_description: item.long_description,
                    author: item.author,
                    url: item.url,
                    icon_url: item.icon_url,
                    versions: item.versions,
                    content_type: item.content_type.into(),
                })
                .collect(),
        })
    }

    async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError> {
        let result_dto: aether_core_plugin_api::v0::ContentItemDto = self
            .call_plugin(&self.capability.handlers.get_content, content_id)
            .await?;
        Ok(result_dto.into())
    }

    async fn list_versions(
        &self,
        content_id: String,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let result_dto: Vec<ContentVersionDto> = self
            .call_optional(self.capability.handlers.list_version.as_ref(), content_id)
            .await?;
        Ok(result_dto.into_iter().map(Into::into).collect())
    }

    async fn install_atomic(
        &self,
        install_params: &AtomicInstallParams,
    ) -> Result<DownloadedContent, InstanceError> {
        let dto: AtomicInstallParamsDto = install_params.clone().into();
        let result_dto: DownloadedContentDto = self
            .call_plugin(&self.capability.handlers.install_atomic, dto)
            .await?;
        Ok(DownloadedContent {
            metadata: result_dto.metadata.into(),
            temp_path: result_dto.temp_path,
        })
    }

    async fn install_modpack(
        &self,
        install_params: &ModpackInstallParams,
    ) -> Result<(String, Vec<ContentFile>), InstanceError> {
        let dto: ModpackInstallParamsDto = install_params.clone().into();
        let result_dto: (String, Vec<aether_core_plugin_api::v0::ContentFileDto>) = self
            .call_optional(self.capability.handlers.install_modpack.as_ref(), dto)
            .await?;
        let files: Vec<ContentFile> = result_dto.1.into_iter().map(Into::into).collect();
        Ok((result_dto.0, files))
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        let params: PluginCheckCompatibilityParamsDto = PluginCheckCompatibilityParams {
            instances: instances.to_vec(),
            check_params: check_params.clone(),
        }
        .into();

        let result_dto: HashMap<String, aether_core_plugin_api::v0::ContentCompatibilityResultDto> =
            self.call_optional(
                self.capability.handlers.check_compatibility.as_ref(),
                params,
            )
            .await?;
        Ok(result_dto
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    ContentCompatibilityResult {
                        is_compatible: v.is_compatible,
                    },
                )
            })
            .collect())
    }
}
