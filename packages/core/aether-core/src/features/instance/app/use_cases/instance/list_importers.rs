use std::sync::Arc;

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
