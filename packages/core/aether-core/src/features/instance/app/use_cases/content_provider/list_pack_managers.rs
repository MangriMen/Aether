use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::ListPackManagersUseCasePort;
use crate::{
    features::instance::{InstanceError, PackManager, PackManagerCapabilityMetadata},
    shared::capability::domain::{CapabilityEntry, CapabilityRegistry},
};

pub struct ListPackManagersUseCase {
    pack_manager_registry: Arc<dyn CapabilityRegistry<Arc<dyn PackManager>>>,
}

impl ListPackManagersUseCase {
    pub fn new(pack_manager_registry: Arc<dyn CapabilityRegistry<Arc<dyn PackManager>>>) -> Self {
        Self {
            pack_manager_registry,
        }
    }

    pub async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<PackManagerCapabilityMetadata>>, InstanceError> {
        Ok(self
            .pack_manager_registry
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

#[async_trait]
impl ListPackManagersUseCasePort for ListPackManagersUseCase {
    async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<PackManagerCapabilityMetadata>>, InstanceError> {
        self.execute().await
    }
}
