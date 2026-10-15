use async_trait::async_trait;

use crate::features::{
    auth::Credential, instance::InstanceError, process::MinecraftProcessMetadata,
};

#[async_trait]
pub trait InstanceLaunchService: Send + Sync {
    async fn execute(
        &self,
        instance_id: String,
        credentials: Credential,
    ) -> Result<MinecraftProcessMetadata, InstanceError>;
}
