use async_trait::async_trait;
use tokio::process::Command;

use crate::features::instance::InstanceId;
use crate::features::process::domain::{MinecraftProcessMetadata, ProcessError};

#[async_trait]
pub trait ProcessStartService: Send + Sync {
    async fn execute(
        &self,
        instance_id: InstanceId,
        command: Command,
        post_exit_command: String,
    ) -> Result<MinecraftProcessMetadata, ProcessError>;
}
