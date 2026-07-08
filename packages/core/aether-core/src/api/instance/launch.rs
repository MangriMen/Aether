use crate::{
    core::app::AetherContainer,
    features::{auth::Credential, instance::InstanceFeature, process::MinecraftProcessMetadata},
};

#[tracing::instrument]
pub async fn run(instance_id: String) -> crate::Result<MinecraftProcessMetadata> {
    let container = AetherContainer::get();
    Ok(container
        .launch_instance_with_active_account_use_case()
        .execute(instance_id)
        .await?)
}

#[tracing::instrument]
pub async fn run_credentials(
    instance_id: String,
    credentials: Credential,
) -> crate::Result<MinecraftProcessMetadata> {
    let container = AetherContainer::get();
    Ok(container
        .instance_launch_service()
        .execute(instance_id, credentials)
        .await?)
}
