use std::sync::Arc;

use crate::features::instance::app::InstanceFileService;
use crate::features::instance::{
    InstanceError, InstanceStorage, InstanceWatcherService, PackStorage,
};

pub struct RemoveInstanceUseCase<IS, IWS, IFS, PS> {
    storage: Arc<IS>,
    watcher_service: Arc<IWS>,
    file_service: Arc<IFS>,
    pack_storage: Arc<PS>,
}

impl<IS: InstanceStorage, IWS: InstanceWatcherService, IFS: InstanceFileService, PS: PackStorage>
    RemoveInstanceUseCase<IS, IWS, IFS, PS>
{
    pub fn new(
        storage: Arc<IS>,
        watcher_service: Arc<IWS>,
        file_service: Arc<IFS>,
        pack_storage: Arc<PS>,
    ) -> Self {
        Self {
            storage,
            watcher_service,
            file_service,
            pack_storage,
        }
    }

    pub async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        self.watcher_service.unwatch_instance(&instance_id).await?;
        self.pack_storage
            .remove_all_for_instance(&instance_id)
            .await?;
        self.file_service.remove_instance_dir(&instance_id).await?;
        self.storage.remove(&instance_id).await?;
        Ok(())
    }
}
