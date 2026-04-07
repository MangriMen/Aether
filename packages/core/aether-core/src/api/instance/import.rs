use crate::{
    core::domain::LazyLocator,
    features::instance::{
        app::{ImportInstance, ImportInstanceUseCase, ListImportersUseCase},
        ImporterCapabilityMetadata,
    },
    shared::CapabilityEntry,
};

#[tracing::instrument]
pub async fn import(import_instance: ImportInstance) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ImportInstanceUseCase::new(lazy_locator.get_importers_registry().await)
            .execute(import_instance)
            .await?,
    )
}

#[tracing::instrument]
pub async fn list_importers() -> crate::Result<Vec<CapabilityEntry<ImporterCapabilityMetadata>>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListImportersUseCase::new(lazy_locator.get_importers_registry().await)
            .execute()
            .await?,
    )
}
