use async_trait::async_trait;

use crate::features::instance::domain::{InstanceError, InstanceId, UpdaterCapabilityMetadata};

#[async_trait]
pub trait Updater: Send + Sync {
    fn metadata(&self) -> &UpdaterCapabilityMetadata;

    async fn update(&self, instance_id: &InstanceId) -> Result<(), InstanceError>;
}
