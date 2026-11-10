use async_trait::async_trait;

use crate::features::{
    events::ProgressBarId,
    minecraft::{GameDownloadParams, InstallMinecraftParams, MinecraftApplicationError},
};

#[async_trait]
pub trait MinecraftInstallService: Send + Sync {
    async fn execute(
        &self,
        params: InstallMinecraftParams,
        loading_bar: Option<&ProgressBarId>,
        force: bool,
    ) -> Result<(), MinecraftApplicationError>;

    /// Download what the game shares between instances (its version's files and its Java), and
    /// nothing that goes into an instance: what an install of the game then finds in place.
    /// `waiting_bar` mirrors the shared downloads, as for [`Self::execute`].
    async fn download_game(
        &self,
        params: GameDownloadParams,
        waiting_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError>;
}
