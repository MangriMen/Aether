use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    features::instance::{Importer, InstanceError},
    shared::CapabilityRegistry,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstance {
    pub plugin_id: String,
    pub importer_id: String,
    pub path: String,
}

pub struct ImportInstanceUseCase<IR: CapabilityRegistry<Arc<dyn Importer>>> {
    importers_registry: Arc<IR>,
}

impl<IR: CapabilityRegistry<Arc<dyn Importer>>> ImportInstanceUseCase<IR> {
    pub fn new(importers_registry: Arc<IR>) -> Self {
        Self { importers_registry }
    }

    pub async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError> {
        let ImportInstance {
            plugin_id,
            importer_id,
            path,
        } = import_instance;

        let importer = self
            .importers_registry
            .find_by_plugin_and_capability_id(&plugin_id, &importer_id)
            .await
            .map_err(|_| InstanceError::ImporterNotFound {
                importer_id: importer_id.to_owned(),
            })?;

        importer.capability.import(&path).await
    }
}
