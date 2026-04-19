use async_trait::async_trait;

use crate::features::{
    events::ProgressBarId,
    minecraft::{MinecraftDomainError, modded, vanilla},
};

#[async_trait]
pub trait MinecraftDownloader: Send + Sync {
    async fn download_minecraft(
        &self,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError>;

    async fn get_version_info(
        &self,
        version: &vanilla::Version,
        loader: Option<&modded::LoaderVersion>,
        force: Option<bool>,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<vanilla::VersionInfo, MinecraftDomainError>;
}
