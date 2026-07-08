use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::features::instance::app::ports::ImportInstanceUseCasePort;
use crate::{
    features::instance::{Importer, InstanceError},
    shared::capability::domain::CapabilityRegistry,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstance {
    pub plugin_id: String,
    pub importer_id: String,
    pub path: String,
}

pub struct ImportInstanceUseCase {
    importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>,
}

impl ImportInstanceUseCase {
    pub fn new(importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>) -> Self {
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
                importer_id: importer_id.clone(),
            })?;

        importer.capability.import(&path).await
    }
}

#[async_trait]
impl ImportInstanceUseCasePort for ImportInstanceUseCase {
    async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError> {
        self.execute(import_instance).await
    }
}
