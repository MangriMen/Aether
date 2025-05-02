use std::{collections::HashMap, path::Path};

use aether_core::features::{
    events::emit_warning,
    instance::{
        ContentInstallParams, ContentSearchParams, ContentSearchResult, ContentType, EditInstance,
        ImportConfig, Instance, InstanceFile, NewInstance,
    },
    process::MinecraftProcessMetadata,
};
use dashmap::DashMap;
use uuid::Uuid;

use crate::{models::minecraft::InstanceImportDto, AetherLauncherResult};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("instance")
        .invoke_handler(tauri::generate_handler![
            instance_create,
            instance_install,
            instance_update,
            instance_list,
            instance_get,
            instance_edit,
            instance_remove,
            instance_get_contents,
            instance_toggle_disable_content,
            instance_disable_contents,
            instance_enable_contents,
            instance_remove_content,
            instance_remove_contents,
            instance_launch,
            instance_stop,
            instance_import,
            instance_get_import_configs,
            instance_get_content_providers,
            instance_get_content_by_provider,
            instance_install_content,
            instance_get_metadata_field_to_check_installed,
            instance_import_contents
        ])
        .build()
}

#[tauri::command]
pub async fn instance_create(new_instance: NewInstance) -> AetherLauncherResult<()> {
    tokio::spawn(async move {
        if let Err(err) = aether_core::api::instance::create(new_instance).await {
            emit_warning(&format!("Error creating instance {}", err))
                .await
                .unwrap_or_else(|e| {
                    log::error!("Error emitting warning: {}", e);
                });
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn instance_install(id: String, force: bool) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::install(id, force).await?)
}

#[tauri::command]
pub async fn instance_update(id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::update(id).await?)
}

#[tauri::command]
pub async fn instance_import(instance_import_dto: InstanceImportDto) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::import(
        &instance_import_dto.pack_type,
        &instance_import_dto.path,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_get_import_configs() -> AetherLauncherResult<Vec<ImportConfig>> {
    Ok(aether_core::api::instance::get_import_configs().await?)
}

#[tauri::command]
pub async fn instance_list() -> AetherLauncherResult<Vec<Instance>> {
    Ok(aether_core::api::instance::list().await?)
}

#[tauri::command]
pub async fn instance_get(id: String) -> AetherLauncherResult<Instance> {
    Ok(aether_core::api::instance::get(id).await?)
}

#[tauri::command]
pub async fn instance_edit(id: String, edit_instance: EditInstance) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::edit(id, edit_instance).await?)
}

#[tauri::command]
pub async fn instance_remove(id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove(id).await?)
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
pub async fn instance_disable_contents(
    id: String,
    content_paths: Vec<String>,
) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::disable_contents(&id, &content_paths).await?)
}

#[tauri::command]
pub async fn instance_enable_contents(
    id: String,
    content_paths: Vec<String>,
) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::enable_contents(&id, &content_paths).await?)
}

#[tauri::command]
pub async fn instance_remove_content(id: String, content_path: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove_content(&id, &content_path).await?)
}

#[tauri::command]
pub async fn instance_remove_contents(
    id: String,
    content_paths: Vec<String>,
) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove_contents(&id, &content_paths).await?)
}

#[tauri::command]
pub async fn instance_launch(id: String) -> AetherLauncherResult<MinecraftProcessMetadata> {
    Ok(aether_core::api::instance::run(&id).await?)
}

#[tauri::command]
pub async fn instance_stop(uuid: Uuid) -> AetherLauncherResult<()> {
    Ok(aether_core::api::process::kill(uuid).await?)
}

#[tauri::command]
pub async fn instance_get_content_providers() -> AetherLauncherResult<HashMap<String, String>> {
    Ok(aether_core::api::instance::get_content_providers().await?)
}

#[tauri::command]
pub async fn instance_get_content_by_provider(
    payload: ContentSearchParams,
) -> AetherLauncherResult<ContentSearchResult> {
    Ok(aether_core::api::instance::get_content_by_provider(&payload).await?)
}

#[tauri::command]
pub async fn instance_install_content(
    id: String,
    payload: ContentInstallParams,
) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::install_content(&id, &payload).await?)
}

#[tauri::command]
pub async fn instance_get_metadata_field_to_check_installed(
    provider: String,
) -> AetherLauncherResult<String> {
    Ok(aether_core::api::instance::get_metadata_field_to_check_installed(&provider).await?)
}

#[tauri::command]
pub async fn instance_import_contents(
    id: &str,
    paths: Vec<String>,
    content_type: ContentType,
) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::import_contents(
        id,
        content_type,
        paths.iter().map(Path::new).collect(),
    )
    .await?)
}
