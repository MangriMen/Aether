use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::GetContentVersionUseCasePort;
use crate::{
    features::instance::{
        ContentProvider, ContentVersion, InstanceError, app::ContentGetVersionParams,
    },
    shared::capability::domain::CapabilityRegistry,
};

pub struct GetContentVersionUseCase {
    provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
}

impl GetContentVersionUseCase {
    pub fn new(provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(
        &self,
        get_params: ContentGetVersionParams,
    ) -> Result<ContentVersion, InstanceError> {
        let provider_id = &get_params.provider_id;

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.to_string(),
                capability_id: provider_id.capability_id.to_string(),
            })?;

        provider
            .capability
            .get_version(&get_params.version_id)
            .await
    }
}

#[async_trait]
impl GetContentVersionUseCasePort for GetContentVersionUseCase {
    async fn execute(
        &self,
        get_params: ContentGetVersionParams,
    ) -> Result<ContentVersion, InstanceError> {
        self.execute(get_params).await
    }
}
