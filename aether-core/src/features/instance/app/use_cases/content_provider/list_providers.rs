use std::sync::Arc;

use crate::{
    features::instance::{ContentProvider, ContentProviderCapabilityMetadata, InstanceError},
    shared::{CapabilityEntry, CapabilityRegistry},
};

pub struct ListProvidersUseCase<CP>
where
    CP: CapabilityRegistry<Arc<dyn ContentProvider>>,
{
    content_provider_registry: Arc<CP>,
}

impl<CP> ListProvidersUseCase<CP>
where
    CP: CapabilityRegistry<Arc<dyn ContentProvider>>,
{
    pub fn new(content_provider_registry: Arc<CP>) -> Self {
        Self {
            content_provider_registry,
        }
    }

    pub async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<ContentProviderCapabilityMetadata>>, InstanceError> {
        Ok(self
            .content_provider_registry
            .list()
            .await
            .map_err(|_| InstanceError::CapabilityOperationError)?
            .into_iter()
            .map(|entry| CapabilityEntry {
                plugin_id: entry.plugin_id,
                capability: entry.capability.metadata().clone(),
            })
            .collect())
    }
}
