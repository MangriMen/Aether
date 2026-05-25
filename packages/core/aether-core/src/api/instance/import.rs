use crate::{
    core::LazyLocator,
    features::instance::{
        ImporterCapabilityMetadata,
        app::{ImportInstance, ImportInstanceUseCase, ListImportersUseCase},
    },
    shared::CapabilityEntry,
};

#[tracing::instrument]
pub async fn import(import_instance: ImportInstance) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(
        ImportInstanceUseCase::new(locator.get_importers_registry().await)
            .execute(import_instance)
            .await?,
    )
}

#[tracing::instrument]
pub async fn list_importers() -> crate::Result<Vec<CapabilityEntry<ImporterCapabilityMetadata>>> {
    let locator = LazyLocator::get().await?;

    Ok(
        ListImportersUseCase::new(locator.get_importers_registry().await)
            .execute()
            .await?,
    )
}
