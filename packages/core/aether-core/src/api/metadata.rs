use crate::{
    core::LazyLocator,
    features::minecraft::{
        GetLoaderVersionManifestUseCase, GetVersionManifestUseCase, ModLoader,
        VersionManifestService, modded, vanilla,
    },
};

pub async fn get_version_manifest() -> crate::Result<vanilla::VersionManifest> {
    let locator = LazyLocator::get().await?;

    Ok(
        VersionManifestService::execute(&GetVersionManifestUseCase::new(
            locator.get_metadata_storage().await,
        ))
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
