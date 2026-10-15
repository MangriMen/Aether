use async_trait::async_trait;
use tokio::process::Command;

use crate::features::{
    auth::Credential,
    minecraft::{GetMinecraftLaunchCommandParams, LaunchSettings, MinecraftApplicationError},
};

#[async_trait]
pub trait MinecraftLaunchCommandService: Send + Sync {
    async fn execute(
        &self,
        params: GetMinecraftLaunchCommandParams,
        launch_settings: LaunchSettings,
        credentials: Credential,
    ) -> Result<Command, MinecraftApplicationError>;
}
