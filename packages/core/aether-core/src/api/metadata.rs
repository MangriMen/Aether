use crate::{
    core::domain::LazyLocator,
    features::minecraft::{
        app::{GetLoaderVersionManifestUseCase, GetVersionManifestUseCase},
        ModLoader,
    },
};

pub async fn get_version_manifest() -> crate::Result<daedalus::minecraft::VersionManifest> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetVersionManifestUseCase::new(lazy_locator.get_metadata_storage().await)
            .execute()
            .await?,
    )
}

pub async fn get_loader_version_manifest(
    loader: ModLoader,
) -> crate::Result<daedalus::modded::Manifest> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetLoaderVersionManifestUseCase::new(lazy_locator.get_metadata_storage().await)
            .execute(loader)
            .await?,
    )
}
