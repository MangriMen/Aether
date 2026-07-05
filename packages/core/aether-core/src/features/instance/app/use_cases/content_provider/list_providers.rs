use std::sync::Arc;

use crate::{
    features::instance::{ContentProvider, ContentProviderCapabilityMetadata, InstanceError},
    shared::capability::domain::{CapabilityEntry, CapabilityRegistry},
};

pub struct ListProvidersUseCase {
    content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
}

impl ListProvidersUseCase {
    pub fn new(
        content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    ) -> Self {
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
