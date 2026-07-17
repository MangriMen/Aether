use std::collections::HashMap;
use std::sync::Arc;

use aether_core_plugin_api::v0::{
    ContentSearchParamsDto, ContentSearchResultDto, ContentVersionDto, ContentVersionIdentifierDto,
    VersionInfoDto,
};
use async_trait::async_trait;
use extism_convert::Msgpack;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;

use crate::features::instance::{
    ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentItem, ContentSearchParams,
    ContentSearchResult, ContentSource, ContentSourceCapabilityMetadata, ContentVersion, Instance,
    InstanceError, VersionInfo, ModpackPayload, VersionPayload, DownloadInstruction, Checksum,
    ContentType, ProviderId,
};
use crate::features::plugins::domain::PluginInstance;
use crate::features::plugins::infra::extism::models::PluginInstanceExt;
use crate::features::plugins::{ContentSourceHandlers, PluginCheckCompatibilityParams};

/// Bridges a WASM plugin's `ContentSource` capability to the domain `ContentSource` trait.
///
/// Each method serializes parameters via `MessagePack`, calls the corresponding
/// WASM function on the plugin, and deserializes the result back.
pub struct PluginContentSourceProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    metadata: ContentSourceCapabilityMetadata,
    handlers: ContentSourceHandlers,
}

impl PluginContentSourceProxy {
    pub fn new(
        instance: Arc<Mutex<dyn PluginInstance>>,
        metadata: ContentSourceCapabilityMetadata,
        handlers: ContentSourceHandlers,
    ) -> Self {
        Self {
            instance,
            metadata,
            handlers,
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
                self.metadata.id
            );
            return Err(InstanceError::ContentProviderNotFound {
                plugin_id,
                capability_id: self.metadata.id.clone(),
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
                self.metadata.id,
                plugin_id
            );

            Err(InstanceError::ContentProviderNotFound {
                plugin_id,
                capability_id: self.metadata.id.clone(),
            })
        }
    }
}

#[async_trait]
impl ContentSource for PluginContentSourceProxy {
    fn metadata(&self) -> &ContentSourceCapabilityMetadata {
        &self.metadata
    }

    async fn search(
        &self,
        search_content: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let dto: ContentSearchParamsDto = search_content.into();
        let result_dto: ContentSearchResultDto = self
            .call_plugin(&self.handlers.search, dto)
            .await?;
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
            .call_plugin(&self.handlers.get_content, content_id)
            .await?;
        Ok(result_dto.into())
    }

    async fn list_versions(
        &self,
        content_id: String,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let result_dto: Vec<ContentVersionDto> = self
            .call_optional(self.handlers.list_version.as_ref(), content_id)
            .await?;
        Ok(result_dto.into_iter().map(Into::into).collect())
    }

    async fn get_version_info(
        &self,
        content_id: &str,
        version_id: &str,
    ) -> Result<VersionInfo, InstanceError> {
        let params = ContentVersionIdentifierDto {
            content_id: content_id.to_owned(),
            version_id: version_id.to_owned(),
        };

        let result_dto: VersionInfoDto = self
            .call_plugin(&self.handlers.get_version_info, params)
            .await?;

        Ok(convert_version_info(result_dto))
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        let params: aether_core_plugin_api::v0::PluginCheckCompatibilityParamsDto =
            PluginCheckCompatibilityParams {
                instances: instances.to_vec(),
                check_params: check_params.clone(),
            }
            .into();

        let result_dto: HashMap<String, aether_core_plugin_api::v0::ContentCompatibilityResultDto> =
            self.call_optional(self.handlers.check_compatibility.as_ref(), params)
                .await?;
        Ok(result_dto
            .into_iter()
            .map(|(k, v)| (k, ContentCompatibilityResult { is_compatible: v.is_compatible }))
            .collect())
    }
}

/// Convert `VersionInfoDto` (from plugin IPC) to domain `VersionInfo`.
#[allow(clippy::cast_sign_loss, clippy::map_unwrap_or)]
fn convert_version_info(dto: VersionInfoDto) -> VersionInfo {
    use crate::features::minecraft::ModLoader;

    let payload = if dto.payload_type == "modpack" {
        VersionPayload::Modpack(Box::new(ModpackPayload {
            format_id: dto.format_id.unwrap_or_default(),
            manifest_bytes: dto.manifest_bytes.unwrap_or_default(),
        }))
    } else {
        VersionPayload::Asset(Box::new(DownloadInstruction {
            url: dto.download_url.unwrap_or_default(),
            file_name: dto.file_name.unwrap_or_default(),
            checksum: dto.sha1.map(|sha| Checksum {
                algorithm: "sha1".to_string(),
                value: sha,
            }),
            headers: None,
            content_path: std::path::PathBuf::default(),
            content_type: ContentType::Mod,
            content_id: String::new(),
            content_version: dto.version_id.clone(),
            provider_id: ProviderId {
                plugin_id: String::new(),
                capability_id: String::new(),
            },
            name: None,
            size: dto.size.map(|s| s as u64).unwrap_or(0),
        }))
    };

    VersionInfo {
        version_id: dto.version_id,
        version_name: dto.version_name,
        game_versions: dto.game_versions,
        loaders: dto
            .loaders
            .into_iter()
            .filter_map(|l| match l.as_str() {
                "fabric" => Some(ModLoader::Fabric),
                "forge" => Some(ModLoader::Forge),
                "neoforge" => Some(ModLoader::NeoForge),
                "quilt" => Some(ModLoader::Quilt),
                "vanilla" => Some(ModLoader::Vanilla),
                _ => None,
            })
            .collect(),
        payload,
    }
}
