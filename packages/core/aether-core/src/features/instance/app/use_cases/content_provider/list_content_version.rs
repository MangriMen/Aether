use std::sync::Arc;

use crate::{
    features::instance::{
        ContentProvider, ContentVersion, InstanceError, app::ContentListVersionsParams,
    },
    shared::CapabilityRegistry,
};

pub struct ListContentVersionsUseCase<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> {
    provider_registry: Arc<CP>,
}

impl<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> ListContentVersionsUseCase<CP> {
    pub fn new(provider_registry: Arc<CP>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(
        &self,
        get_params: ContentListVersionsParams,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let provider_id = &get_params.provider_id;

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.clone(),
                capability_id: provider_id.capability_id.clone(),
            })?;

        provider
            .capability
            .list_versions(get_params.content_id)
            .await
    }
}
