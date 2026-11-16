use async_trait::async_trait;
use tokio::process::Command;

use crate::features::instance::InstanceId;
use crate::features::process::domain::{KillScope, MinecraftProcessMetadata, ProcessError};

#[async_trait]
pub trait ProcessStartService: Send + Sync {
    async fn execute(
        &self,
        instance_id: InstanceId,
        command: Command,
        kill_scope: KillScope,
        post_exit_command: String,
        secrets: Vec<String>,
    ) -> Result<MinecraftProcessMetadata, ProcessError>;
}
