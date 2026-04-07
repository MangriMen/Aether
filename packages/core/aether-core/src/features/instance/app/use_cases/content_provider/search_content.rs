use std::sync::Arc;

use crate::{
    features::instance::{
        ContentProvider, ContentSearchParams, ContentSearchResult, InstanceError,
    },
    shared::CapabilityRegistry,
};

pub struct SearchContentUseCase<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> {
    provider_registry: Arc<CP>,
}

impl<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> SearchContentUseCase<CP> {
    pub fn new(provider_registry: Arc<CP>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(
        &self,
        search_params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let provider_id = &search_params.provider_id;

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.to_string(),
                capability_id: provider_id.capability_id.to_string(),
            })?;

        provider.capability.search(search_params).await
    }
}
