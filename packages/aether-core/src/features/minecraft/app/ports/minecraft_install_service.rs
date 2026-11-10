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

/// Stands in for the installs of the game in tests of what runs around them.
#[cfg(test)]
#[derive(Default)]
pub struct RecordingGameDownload {
    /// The game versions asked for, in order.
    pub downloads: std::sync::Mutex<Vec<crate::features::minecraft::GameVersion>>,
    /// Set it and a download never finishes: the one of a game the user calls off.
    pub hangs: std::sync::atomic::AtomicBool,
    /// Told when a download has begun.
    pub started: tokio::sync::Notify,
}

#[cfg(test)]
#[async_trait]
impl MinecraftInstallService for RecordingGameDownload {
    async fn execute(
        &self,
        _: InstallMinecraftParams,
        _: Option<&ProgressBarId>,
        _: bool,
    ) -> Result<(), MinecraftApplicationError> {
        unimplemented!("a plan installs the game through its instance")
    }

    async fn download_game(
        &self,
        params: GameDownloadParams,
        _: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError> {
        self.downloads.lock().unwrap().push(params.game_version);
        self.started.notify_one();
        if self.hangs.load(std::sync::atomic::Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        Ok(())
    }
}
