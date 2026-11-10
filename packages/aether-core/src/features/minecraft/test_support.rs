//! Test doubles of the ports of this feature that tests of other features share.
use async_trait::async_trait;

use crate::features::{
    events::ProgressBarId,
    minecraft::{
        GameDownloadParams, GameVersion, InstallMinecraftParams, MinecraftApplicationError,
        MinecraftInstallService,
    },
};

/// Stands in for the installs of the game in tests of what runs around them.
#[derive(Default)]
pub(crate) struct RecordingGameDownload {
    /// The game versions asked for, in order.
    pub(crate) downloads: std::sync::Mutex<Vec<GameVersion>>,
    /// Set it and a download never finishes: the one of a game the user calls off.
    pub(crate) hangs: std::sync::atomic::AtomicBool,
    /// Told when a download has begun.
    pub(crate) started: tokio::sync::Notify,
}

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
