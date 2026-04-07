use std::sync::Arc;

use crate::{
    features::instance::{Importer, ImporterCapabilityMetadata, InstanceError},
    shared::{CapabilityEntry, CapabilityRegistry},
};

pub struct ListImportersUseCase<IR: CapabilityRegistry<Arc<dyn Importer>>> {
    importers_registry: Arc<IR>,
}

impl<IR: CapabilityRegistry<Arc<dyn Importer>>> ListImportersUseCase<IR> {
    pub fn new(importers_registry: Arc<IR>) -> Self {
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
