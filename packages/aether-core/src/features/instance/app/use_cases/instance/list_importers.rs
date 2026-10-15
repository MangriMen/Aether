use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::ListImportersUseCasePort;
use crate::{
    features::instance::{Importer, ImporterCapabilityMetadata, InstanceError},
    shared::capability::domain::{CapabilityEntry, CapabilityRegistry},
};

pub struct ListImportersUseCase {
    importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>,
}

impl ListImportersUseCase {
    pub fn new(importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>) -> Self {
        Self { importers_registry }
    }

    pub async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<ImporterCapabilityMetadata>>, InstanceError> {
        Ok(self
            .importers_registry
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
impl ListImportersUseCasePort for ListImportersUseCase {
    async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<ImporterCapabilityMetadata>>, InstanceError> {
        self.execute().await
    }
}
