use std::path::Path;

use async_trait::async_trait;
use daedalus::minecraft::VersionInfo;

use crate::features::{events::ProgressBarId, java::Java, minecraft::MinecraftDomainError};

#[async_trait]
pub trait ModLoaderProcessor {
    async fn run(
        &self,
        game_version: String,
        version_jar: String,
        minecraft_path: &Path,
        version_info: &mut VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError>;
}
