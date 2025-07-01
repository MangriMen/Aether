use dashmap::DashMap;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

use aether_core::features::{
    instance::{
        ContentInstallParams, ContentSearchParams, ContentSearchResult, ContentType, EditInstance,
        ImportConfig, Instance, InstanceFile, NewInstance,
    },
    process::MinecraftProcessMetadata,
};

use crate::FrontendResult;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceImportDto {
    pub pack_type: String,
    pub path: String,
}

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
            instance_import_contents,
            instance_get_dir,
        ])
        .build()
}

#[tauri::command]
pub async fn instance_create(new_instance: NewInstance) -> FrontendResult<()> {
    tokio::spawn(async move {
        if let Err(err) = aether_core::api::instance::create(new_instance).await {
            debug!("{:?}", err)
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn instance_install(id: String, force: bool) -> FrontendResult<()> {
    Ok(aether_core::api::instance::install(id, force).await?)
}

#[tauri::command]
pub async fn instance_update(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::instance::update(id).await?)
}

#[tauri::command]
pub async fn instance_import(instance_import_dto: InstanceImportDto) -> FrontendResult<()> {
    Ok(aether_core::api::instance::import(
        &instance_import_dto.pack_type,
        &instance_import_dto.path,
    )
    .await?)
}

#[tauri::command]
pub async fn instance_get_import_configs() -> FrontendResult<Vec<ImportConfig>> {
    Ok(aether_core::api::instance::get_import_configs().await?)
}

#[tauri::command]
pub async fn instance_list() -> FrontendResult<Vec<Instance>> {
    Ok(aether_core::api::instance::list().await?)
}

#[tauri::command]
pub async fn instance_get(id: String) -> FrontendResult<Instance> {
    Ok(aether_core::api::instance::get(id).await?)
}

#[tauri::command]
pub async fn instance_edit(id: String, edit_instance: EditInstance) -> FrontendResult<()> {
    Ok(aether_core::api::instance::edit(id, edit_instance).await?)
}

#[tauri::command]
pub async fn instance_remove(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::instance::remove(id).await?)
}

#[tauri::command]
pub async fn instance_get_contents(id: String) -> FrontendResult<DashMap<String, InstanceFile>> {
    Ok(aether_core::api::instance::get_contents(id).await?)
}

#[tauri::command]
pub async fn instance_disable_contents(
    id: String,
    content_paths: Vec<String>,
) -> FrontendResult<()> {
    Ok(aether_core::api::instance::disable_contents(id, content_paths).await?)
}

#[tauri::command]
pub async fn instance_enable_contents(
    id: String,
    content_paths: Vec<String>,
) -> FrontendResult<()> {
    Ok(aether_core::api::instance::enable_contents(id, content_paths).await?)
}

#[tauri::command]
pub async fn instance_remove_content(id: String, content_path: String) -> FrontendResult<()> {
    Ok(aether_core::api::instance::remove_content(id, content_path).await?)
}

#[tauri::command]
pub async fn instance_remove_contents(
    id: String,
    content_paths: Vec<String>,
) -> FrontendResult<()> {
    Ok(aether_core::api::instance::remove_contents(id, content_paths).await?)
}

#[tauri::command]
pub async fn instance_launch(id: String) -> FrontendResult<MinecraftProcessMetadata> {
    Ok(aether_core::api::instance::run(id).await?)
}

#[tauri::command]
pub async fn instance_stop(uuid: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::process::kill(uuid).await?)
}

#[tauri::command]
pub async fn instance_get_content_providers() -> FrontendResult<HashMap<String, String>> {
    Ok(aether_core::api::instance::get_content_providers().await?)
}

#[tauri::command]
pub async fn instance_get_content_by_provider(
    payload: ContentSearchParams,
) -> FrontendResult<ContentSearchResult> {
    Ok(aether_core::api::instance::search_content(payload).await?)
}

#[tauri::command]
pub async fn instance_install_content(
    id: String,
    payload: ContentInstallParams,
) -> FrontendResult<()> {
    Ok(aether_core::api::instance::install_content(id, payload).await?)
}

#[tauri::command]
pub async fn instance_get_metadata_field_to_check_installed(
    provider: String,
) -> FrontendResult<String> {
    Ok(aether_core::api::instance::get_metadata_field_to_check_installed(provider).await?)
}

#[tauri::command]
pub async fn instance_import_contents(
    instance_id: String,
    content_type: ContentType,
    source_paths: Vec<String>,
) -> FrontendResult<()> {
    Ok(aether_core::api::instance::import_contents(
        instance_id,
        content_type,
        source_paths.iter().map(PathBuf::from).collect(),
    )
    .await?)
}

#[tauri::command]
pub async fn instance_get_dir(id: String) -> FrontendResult<PathBuf> {
    Ok(aether_core::api::instance::get_dir(&id).await?)
}
