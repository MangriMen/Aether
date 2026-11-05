use async_trait::async_trait;
use std::path::Path;

use crate::features::{
    events::ProgressBarId,
    java::Java,
    minecraft::{GameVersion, MinecraftDomainError, vanilla},
};

#[async_trait]
pub trait ModLoaderProcessor: Send + Sync {
    async fn run(
        &self,
        game_version: GameVersion,
        version_jar: String,
        minecraft_path: &Path,
        version_info: &mut vanilla::VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError>;
}
