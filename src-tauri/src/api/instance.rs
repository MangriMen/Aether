use aether_core::state::{ImportHandler, Instance, InstanceFile, MinecraftProcessMetadata};
use dashmap::DashMap;
use uuid::Uuid;

use crate::{
    models::minecraft::{InstanceCreateDto, InstanceEditDto, InstanceImportDto},
    AetherLauncherResult,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("instance")
        .invoke_handler(tauri::generate_handler![
            instance_create,
            instance_list,
            instance_get,
            instance_edit,
            instance_remove,
            instance_get_contents,
            instance_toggle_disable_content,
            instance_remove_content,
            instance_launch,
            instance_stop,
            instance_get_import_handlers
        ])
        .build()
}

#[tauri::command]
pub async fn instance_create(instance_create_dto: InstanceCreateDto) -> AetherLauncherResult<()> {
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
pub async fn instance_import(instance_import_dto: InstanceImportDto) -> AetherLauncherResult<()> {
    todo!()
}

#[tauri::command]
pub async fn instance_get_import_handlers() -> AetherLauncherResult<Vec<ImportHandler>> {
    Ok(aether_core::api::instance::get_import_handlers().await?)
}

#[tauri::command]
pub async fn instance_list() -> AetherLauncherResult<(Vec<Instance>, Vec<String>)> {
    let res = aether_core::api::instance::get_all().await?;
    Ok((res.0, res.1.iter().map(|err| err.to_string()).collect()))
}

#[tauri::command]
pub async fn instance_get(id: String) -> AetherLauncherResult<Instance> {
    Ok(aether_core::api::instance::get(&id).await?)
}

#[tauri::command]
pub async fn instance_edit(
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
pub async fn instance_remove(id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove(&id).await?)
}

#[tauri::command]
pub async fn instance_get_contents(
    id: String,
) -> AetherLauncherResult<DashMap<String, InstanceFile>> {
    Ok(aether_core::api::instance::get_contents(&id).await?)
}

#[tauri::command]
pub async fn instance_toggle_disable_content(
    id: String,
    content_path: String,
) -> AetherLauncherResult<String> {
    Ok(aether_core::api::instance::toggle_disable_content(&id, &content_path).await?)
}

#[tauri::command]
pub async fn instance_remove_content(id: String, content_path: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove_content(&id, &content_path).await?)
}

#[tauri::command]
pub async fn instance_launch(id: String) -> AetherLauncherResult<MinecraftProcessMetadata> {
    Ok(aether_core::api::instance::run(&id).await?)
}

#[tauri::command]
pub async fn instance_stop(uuid: Uuid) -> AetherLauncherResult<()> {
    Ok(aether_core::api::process::kill(uuid).await?)
}
