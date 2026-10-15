use async_trait::async_trait;

use crate::features::instance::InstanceError;

#[async_trait]
pub trait InstanceInstallService: Send + Sync {
    async fn execute(&self, instance_id: String, force: bool) -> Result<(), InstanceError>;
}
