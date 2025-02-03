use aether_core::state::{Instance, MinecraftProcessMetadata};
use uuid::Uuid;

use crate::{
    models::minecraft::{InstanceCreateDto, InstanceEditDto},
    AetherLauncherResult,
};

#[tauri::command]
pub async fn create_minecraft_instance(
    instance_create_dto: InstanceCreateDto,
) -> AetherLauncherResult<()> {
    tokio::spawn(async move {
        aether_core::api::instance::create(
            instance_create_dto.name,
            instance_create_dto.game_version,
            instance_create_dto.mod_loader,
            instance_create_dto.loader_version,
            instance_create_dto.icon_path,
            instance_create_dto.skip_install_profile,
            None,
        )
        .await
    });

    Ok(())
}

#[tauri::command]
pub async fn edit_minecraft_instance(
    id: String,
    instance_edit_dto: InstanceEditDto,
) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::edit(
        &id,
        &instance_edit_dto.name,
        &instance_edit_dto.java_path,
        &instance_edit_dto.extra_launch_args,
        &instance_edit_dto.custom_env_vars,
        &instance_edit_dto.memory,
        &instance_edit_dto.game_resolution,
    )
    .await?)
}

#[tauri::command]
pub async fn remove_minecraft_instance(id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove(&id).await?)
}

#[tauri::command]
pub async fn launch_minecraft_instance(
    id: String,
) -> AetherLauncherResult<MinecraftProcessMetadata> {
    Ok(aether_core::api::instance::run(&id).await?)
}

#[tauri::command]
pub async fn stop_minecraft_instance(uuid: Uuid) -> AetherLauncherResult<()> {
    Ok(aether_core::api::process::kill(uuid).await?)
}

#[tauri::command]
pub async fn get_minecraft_instances() -> AetherLauncherResult<(Vec<Instance>, Vec<String>)> {
    let res = aether_core::api::instance::get_all().await?;
    Ok((res.0, res.1.iter().map(|err| err.to_string()).collect()))
}
