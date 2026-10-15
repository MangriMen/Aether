use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::features::instance::app::ports::ImportInstanceUseCasePort;
use crate::{
    features::instance::{
        Importer, InstanceError, InstanceInstallStage, InstanceStorage,
        app::extensions::InstanceStorageExt,
    },
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
    instance_storage: Arc<dyn InstanceStorage>,
}

impl ImportInstanceUseCase {
    pub fn new(
        importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>,
        instance_storage: Arc<dyn InstanceStorage>,
    ) -> Self {
        Self {
            importers_registry,
            instance_storage,
        }
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

        // A failed import leaves its pack instances `PackInstalling`: they are not complete.
        let pack_instances = importer.capability.import(&path).await?;

        for instance_id in pack_instances {
            self.instance_storage
                .upsert_with(&instance_id, |instance| {
                    if instance.install_stage == InstanceInstallStage::PackInstalling {
                        instance.install_stage = InstanceInstallStage::Installed;
                    }
                    Ok(())
                })
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl ImportInstanceUseCasePort for ImportInstanceUseCase {
    async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError> {
        self.execute(import_instance).await
    }
}
