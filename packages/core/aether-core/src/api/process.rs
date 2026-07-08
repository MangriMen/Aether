use uuid::Uuid;

use crate::{
    core::app::AetherContainer,
    features::process::{MinecraftProcessMetadata, ProcessFeature},
};

pub async fn list() -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let container = AetherContainer::get();
    Ok(container.list_process_metadata_use_case().execute().await?)
}

#[tracing::instrument]
pub async fn get_by_instance_id(
    instance_id: String,
) -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let container = AetherContainer::get();
    Ok(container
        .get_process_metadata_by_instance_id_use_case()
        .execute(instance_id)
        .await?)
}

#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container.kill_process_use_case().execute(uuid).await?)
}

pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container.wait_for_process_use_case().execute(uuid).await?)
}
