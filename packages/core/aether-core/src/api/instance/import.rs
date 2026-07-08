use crate::{
    core::app::AetherContainer,
    features::instance::{ImportInstance, ImporterCapabilityMetadata, InstanceFeature},
    shared::capability::domain::CapabilityEntry,
};

#[tracing::instrument]
pub async fn import(import_instance: ImportInstance) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .import_instance_use_case()
        .execute(import_instance)
        .await?)
}

#[tracing::instrument]
pub async fn list_importers() -> crate::Result<Vec<CapabilityEntry<ImporterCapabilityMetadata>>> {
    let container = AetherContainer::get();
    Ok(container.list_importers_use_case().execute().await?)
}
