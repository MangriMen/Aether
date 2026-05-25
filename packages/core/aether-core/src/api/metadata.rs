use crate::{
    core::LazyLocator,
    features::minecraft::{
        ModLoader,
        app::{GetLoaderVersionManifestUseCase, GetVersionManifestUseCase},
        modded, vanilla,
    },
};

pub async fn get_version_manifest() -> crate::Result<vanilla::VersionManifest> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetVersionManifestUseCase::new(locator.get_metadata_storage().await)
            .execute()
            .await?,
    )
}

pub async fn get_loader_version_manifest(loader: ModLoader) -> crate::Result<modded::Manifest> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetLoaderVersionManifestUseCase::new(locator.get_metadata_storage().await)
            .execute(loader)
            .await?,
    )
}
