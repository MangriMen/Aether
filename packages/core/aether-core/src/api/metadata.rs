use crate::{
    core::app::AetherContainer,
    features::minecraft::{MinecraftFeature, ModLoader, modded, vanilla},
};

pub async fn get_version_manifest() -> crate::Result<vanilla::VersionManifest> {
    let container = AetherContainer::get();
    Ok(container.get_version_manifest_use_case().execute().await?)
}

pub async fn get_loader_version_manifest(loader: ModLoader) -> crate::Result<modded::Manifest> {
    let container = AetherContainer::get();
    Ok(container
        .get_loader_version_manifest_use_case()
        .execute(loader)
        .await?)
}
