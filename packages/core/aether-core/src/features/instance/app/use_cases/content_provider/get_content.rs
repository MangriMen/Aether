use std::sync::Arc;

use crate::{
    features::instance::{ContentItem, ContentProvider, InstanceError, app::ContentGetParams},
    shared::CapabilityRegistry,
};

pub struct GetContentUseCase<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> {
    provider_registry: Arc<CP>,
}

impl<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> GetContentUseCase<CP> {
    pub fn new(provider_registry: Arc<CP>) -> Self {
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
