use std::sync::Arc;

use crate::{
    features::instance::{
        InstanceError, InstanceInstallStage, InstanceStorage, InstanceStorageExt, PackInfo, Updater,
    },
    shared::CapabilityRegistry,
};

pub struct UpdateInstanceUseCase<IS: InstanceStorage, UR: CapabilityRegistry<Arc<dyn Updater>>> {
    instance_storage: Arc<IS>,
    updaters_registry: Arc<UR>,
}

impl<IS: InstanceStorage, UR: CapabilityRegistry<Arc<dyn Updater>>> UpdateInstanceUseCase<IS, UR> {
    pub fn new(instance_storage: Arc<IS>, updaters_registry: Arc<UR>) -> Self {
        Self {
            instance_storage,
            updaters_registry,
        }
    }

    pub async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        let instance = self.instance_storage.get(&instance_id).await?;

        let original_stage = instance.install_stage;
        let pack_info = instance
            .pack_info
            .as_ref()
            .ok_or(InstanceError::PackInfoNotFound)?;

        self.instance_storage
            .upsert_with(&instance_id, |instance| {
                instance.install_stage = InstanceInstallStage::PackInstalling;
                Ok(())
            })
            .await?;

        let result = self.update_by_plugin(&instance_id, pack_info).await;

        let final_stage = match result {
            Ok(_) => InstanceInstallStage::Installed,
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

    pub async fn update_by_plugin(
        &self,
        instance_id: &str,
        pack_info: &PackInfo,
    ) -> Result<(), InstanceError> {
        let modpack_id = &pack_info.modpack_id;
        let plugin_id = &pack_info.plugin_id;

        let updater = self
            .updaters_registry
            .find_by_plugin_and_capability_id(plugin_id, modpack_id)
            .await
            .map_err(|_| InstanceError::UpdaterNotFound {
                modpack_id: modpack_id.to_string(),
            })?;

        updater.capability.update(instance_id).await
    }
}
