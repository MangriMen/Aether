use async_trait::async_trait;

use crate::features::instance::domain::{InstanceError, InstanceId};

#[async_trait]
pub trait InstanceWatcherService: Send + Sync {
    async fn watch_instances(&self) -> Result<(), InstanceError>;
    async fn watch_instance(&self, instance_id: &InstanceId) -> Result<(), InstanceError>;
    async fn unwatch_instance(&self, instance_id: &InstanceId) -> Result<(), InstanceError>;
}
