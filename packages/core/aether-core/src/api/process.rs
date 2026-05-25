use uuid::Uuid;

use crate::{
    core::LazyLocator,
    features::process::{
        MinecraftProcessMetadata,
        app::{
            GetProcessMetadataByInstanceIdUseCase, KillProcessUseCase, ListProcessMetadataUseCase,
            WaitForProcessUseCase,
        },
    },
};

pub async fn list() -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let locator = LazyLocator::get().await?;

    Ok(
        ListProcessMetadataUseCase::new(locator.get_process_storage().await)
            .execute()
            .await?,
    )
}

#[tracing::instrument]
pub async fn get_by_instance_id(
    instance_id: String,
) -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetProcessMetadataByInstanceIdUseCase::new(locator.get_process_storage().await)
            .execute(instance_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(KillProcessUseCase::new(locator.get_process_storage().await)
        .execute(uuid)
        .await?)
}

pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(
        WaitForProcessUseCase::new(locator.get_process_storage().await)
            .execute(uuid)
            .await?,
    )
}
