use async_trait::async_trait;

use crate::features::minecraft::{
    GameVersion, LoaderVersionPreference, MinecraftDomainError, ModLoader, modded,
};

#[async_trait]
pub trait LoaderVersionService: Send + Sync {
    async fn resolve(
        &self,
        game_version: &GameVersion,
        loader: &ModLoader,
        loader_version: Option<&LoaderVersionPreference>,
    ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError>;

    async fn try_get_default(
        &self,
        game_version: &GameVersion,
        loader: &ModLoader,
    ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError>;
}
