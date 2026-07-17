use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::features::instance::domain::InstanceError;
use crate::features::instance::{
    ContentItem, ContentSearchParams, ContentSearchResult, ContentSource,
    ContentSourceCapabilityMetadata, ContentVersion, VersionInfo,
};
use crate::features::plugins::domain::PluginInstance;
use crate::features::plugins::ContentSourceHandlers;

/// Bridges a WASM plugin's `ContentSource` capability to the domain `ContentSource` trait.
///
/// Each method serializes parameters via MessagePack, calls the corresponding
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
}

#[async_trait]
impl ContentSource for PluginContentSourceProxy {
    fn metadata(&self) -> &ContentSourceCapabilityMetadata {
        &self.metadata
    }

    async fn search(
        &self,
        _params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        Err(InstanceError::UnsupportedOperation("search".into()))
    }

    async fn get_content(&self, _content_id: String) -> Result<ContentItem, InstanceError> {
        Err(InstanceError::UnsupportedOperation("get_content".into()))
    }

    async fn list_versions(
        &self,
        _content_id: String,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        Err(InstanceError::UnsupportedOperation(
            "list_versions".into(),
        ))
    }

    async fn get_version_info(
        &self,
        _content_id: &str,
        _version_id: &str,
    ) -> Result<VersionInfo, InstanceError> {
        Err(InstanceError::UnsupportedOperation(
            "get_version_info".into(),
        ))
    }

    async fn check_compatibility(
        &self,
        _instances: &[crate::features::instance::Instance],
        _content_item: &crate::features::instance::app::ContentCompatibilityCheckParams,
    ) -> Result<
        HashMap<String, crate::features::instance::app::ContentCompatibilityResult>,
        InstanceError,
    > {
        Err(InstanceError::UnsupportedOperation(
            "check_compatibility".into(),
        ))
    }
}
