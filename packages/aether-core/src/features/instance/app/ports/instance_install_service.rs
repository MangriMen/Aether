use async_trait::async_trait;

use crate::features::instance::{InstanceError, InstanceId};

#[async_trait]
pub trait InstanceInstallService: Send + Sync {
    async fn execute(&self, instance_id: InstanceId, force: bool) -> Result<(), InstanceError>;
}
