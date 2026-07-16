use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::UpdateInstanceUseCasePort;
use crate::{
    features::instance::{
        DownloadContext, InstanceError, InstanceInstallStage, InstanceStorage, InstanceStorageExt,
        PackManager,
    },
    shared::capability::domain::CapabilityRegistry,
};

/// Updates a pack-managed instance via the `PackManager` registered for its
/// `pack_info.provider_id`.
pub struct UpdateInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    pack_manager_registry: Arc<dyn CapabilityRegistry<Arc<dyn PackManager>>>,
}

impl UpdateInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        pack_manager_registry: Arc<dyn CapabilityRegistry<Arc<dyn PackManager>>>,
    ) -> Self {
        Self {
            instance_storage,
            pack_manager_registry,
        }
    }

    pub async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        let instance = self.instance_storage.get(&instance_id).await?;

        let original_stage = instance.install_stage;
        let pack_info = instance
            .pack_info
            .as_ref()
            .ok_or(InstanceError::PackInfoNotFound)?;

        let provider_id = &pack_info.provider_id;

        let manager = self
            .pack_manager_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::PackManagerNotFound {
                plugin_id: provider_id.plugin_id.clone(),
                capability_id: provider_id.capability_id.clone(),
            })?;

        self.instance_storage
            .upsert_with(&instance_id, |instance| {
                instance.install_stage = InstanceInstallStage::PackInstalling;
                Ok(())
            })
            .await?;

        let result = manager
            .capability
            .update(&instance_id, &DownloadContext {})
            .await;

        let final_stage = match result {
            Ok(()) => InstanceInstallStage::Installed,
            Err(_) => original_stage,
        };

        self.instance_storage
            .upsert_with(&instance_id, |instance| {
                instance.install_stage = final_stage;
                Ok(())
            })
            .await?;

        result
    }
}

#[async_trait]
impl UpdateInstanceUseCasePort for UpdateInstanceUseCase {
    async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        self.execute(instance_id).await
    }
}
