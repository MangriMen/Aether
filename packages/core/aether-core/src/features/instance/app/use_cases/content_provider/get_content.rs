use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::GetContentUseCasePort;
use crate::{
    features::instance::{ContentItem, ContentProvider, InstanceError, app::ContentGetParams},
    shared::capability::domain::CapabilityRegistry,
};

pub struct GetContentUseCase {
    provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
}

impl GetContentUseCase {
    pub fn new(provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(
        &self,
        get_params: ContentGetParams,
    ) -> Result<ContentItem, InstanceError> {
        let provider_id = &get_params.provider_id;

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.clone(),
                capability_id: provider_id.capability_id.clone(),
            })?;

        provider.capability.get_content(get_params.content_id).await
    }
}

#[async_trait]
impl GetContentUseCasePort for GetContentUseCase {
    async fn execute(&self, get_params: ContentGetParams) -> Result<ContentItem, InstanceError> {
        self.execute(get_params).await
    }
}
