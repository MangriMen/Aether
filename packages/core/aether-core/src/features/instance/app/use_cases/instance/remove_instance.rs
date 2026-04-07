use std::sync::Arc;

use crate::features::instance::{InstanceError, InstanceStorage, InstanceWatcherService};

pub struct RemoveInstanceUseCase<IS, IWS> {
    instance_storage: Arc<IS>,
    instance_watcher_service: Arc<IWS>,
}

impl<IS: InstanceStorage, IWS: InstanceWatcherService> RemoveInstanceUseCase<IS, IWS> {
    pub fn new(instance_storage: Arc<IS>, instance_watcher_service: Arc<IWS>) -> Self {
        Self {
            instance_storage,
            instance_watcher_service,
        }
    }

    pub async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        self.instance_watcher_service
            .unwatch_instance(&instance_id)
            .await?;
        self.instance_storage.remove(&instance_id).await?;
        Ok(())
    }
}
