use async_trait::async_trait;

use crate::features::{
    events::ProgressBarId,
    minecraft::{MinecraftDomainError, modded, vanilla},
};

#[async_trait]
pub trait MinecraftDownloader: Send + Sync {
    /// Download what is missing of the files of `version_info` that its instances share (the
    /// client, the libraries, the assets). One download of a version runs at a time, under a
    /// shared bar of its own; an install asking for a version being downloaded waits for it.
    ///
    /// `waiting_bar` is the bar of the install that needs the version: while it waits on the
    /// download (its own or another's), the bar has a mirror of the download's bar. It gets none
    /// when every file is there already.
    async fn download_minecraft(
        &self,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        waiting_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError>;

    async fn get_version_info(
        &self,
        version: &vanilla::Version,
        loader: Option<&modded::LoaderVersion>,
        force: Option<bool>,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<vanilla::VersionInfo, MinecraftDomainError>;
}
