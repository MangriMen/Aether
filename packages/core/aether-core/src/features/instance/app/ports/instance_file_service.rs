use async_trait::async_trait;

use crate::features::instance::domain::InstanceError;

#[async_trait]
pub trait InstanceFileService: Send + Sync {
    async fn create_instance_dir(&self, instance_id: &str) -> Result<(), InstanceError>;

    async fn remove_instance_dir(&self, instance_id: &str) -> Result<(), InstanceError>;
}
