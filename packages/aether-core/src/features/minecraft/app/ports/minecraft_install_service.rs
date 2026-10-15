use async_trait::async_trait;

use crate::features::{
    events::ProgressBarId,
    minecraft::{InstallMinecraftParams, MinecraftApplicationError},
};

#[async_trait]
pub trait MinecraftInstallService: Send + Sync {
    async fn execute(
        &self,
        params: InstallMinecraftParams,
        loading_bar: Option<&ProgressBarId>,
        force: bool,
    ) -> Result<(), MinecraftApplicationError>;
}
