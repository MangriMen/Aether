use async_trait::async_trait;

use crate::features::instance::{InstanceError, UpdaterCapabilityMetadata};

#[async_trait]
pub trait Updater: Send + Sync {
    fn metadata(&self) -> &UpdaterCapabilityMetadata;

    async fn update(&self, instance_id: &str) -> Result<(), InstanceError>;
}
