use dashmap::DashMap;
use log::debug;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

use aether_core::features::{
    instance::{
        ContentInstallParams, ContentSearchParams, ContentSearchResult, ContentType, EditInstance,
        ImportInstance, Instance, ContentFile, NewInstance,
    },
    process::MinecraftProcessMetadata,
};

use crate::FrontendResult;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("instance")
        .invoke_handler(tauri::generate_handler![
            create,
            import,
            list,
            get,
            get_dir,
            install,
            update,
            edit,
            remove,
            launch,
            stop,
            import_contents,
            list_content,
            install_content,
            enable_contents,
            disable_contents,
            remove_contents,
            get_content_providers,
            get_content_by_provider,
            get_metadata_field_to_check_installed,
        ])
        .build()
}

#[tauri::command]
async fn create(new_instance: NewInstance) -> FrontendResult<()> {
    tokio::spawn(async move {
        if let Err(err) = aether_core::api::instance::create(new_instance).await {
            debug!("{:?}", err)
        }
    });
    Ok(())
}

#[tauri::command]
async fn import(import_instance: ImportInstance) -> FrontendResult<()> {
    Ok(aether_core::api::instance::import(import_instance).await?)
}

#[tauri::command]
async fn list() -> FrontendResult<Vec<Instance>> {
    Ok(aether_core::api::instance::list().await?)
}

#[tauri::command]
async fn get(id: String) -> FrontendResult<Instance> {
    Ok(aether_core::api::instance::get(id).await?)
}

#[tauri::command]
async fn get_dir(id: String) -> FrontendResult<PathBuf> {
    Ok(aether_core::api::instance::get_dir(&id).await?)
}

#[tauri::command]
async fn install(id: String, force: bool) -> FrontendResult<()> {
    Ok(aether_core::api::instance::install(id, force).await?)
}

#[tauri::command]
async fn update(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::instance::update(id).await?)
}

#[tauri::command]
async fn edit(id: String, edit_instance: EditInstance) -> FrontendResult<Instance> {
    Ok(aether_core::api::instance::edit(id, edit_instance).await?)
}

#[tauri::command]
async fn remove(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::instance::remove(id).await?)
}

#[tauri::command]
async fn launch(id: String) -> FrontendResult<MinecraftProcessMetadata> {
    Ok(aether_core::api::instance::run(id).await?)
}

#[tauri::command]
async fn stop(uuid: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::process::kill(uuid).await?)
}

#[tauri::command]
async fn import_contents(
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
async fn list_content(id: String) -> FrontendResult<DashMap<String, ContentFile>> {
    Ok(aether_core::api::instance::list_content(id).await?)
}

#[tauri::command]
async fn install_content(id: String, payload: ContentInstallParams) -> FrontendResult<()> {
    Ok(aether_core::api::instance::install_content(id, payload).await?)
}

#[tauri::command]
async fn enable_contents(id: String, content_paths: Vec<String>) -> FrontendResult<()> {
    Ok(aether_core::api::instance::enable_contents(id, content_paths).await?)
}

#[tauri::command]
async fn disable_contents(id: String, content_paths: Vec<String>) -> FrontendResult<()> {
    Ok(aether_core::api::instance::disable_contents(id, content_paths).await?)
}

#[tauri::command]
async fn remove_contents(id: String, content_paths: Vec<String>) -> FrontendResult<()> {
    Ok(aether_core::api::instance::remove_contents(id, content_paths).await?)
}

#[tauri::command]
async fn get_content_providers() -> FrontendResult<HashMap<String, String>> {
    Ok(aether_core::api::instance::get_content_providers().await?)
}

#[tauri::command]
async fn get_content_by_provider(
    payload: ContentSearchParams,
) -> FrontendResult<ContentSearchResult> {
    Ok(aether_core::api::instance::search_content(payload).await?)
}

#[tauri::command]
async fn get_metadata_field_to_check_installed(provider: String) -> FrontendResult<String> {
    Ok(aether_core::api::instance::get_metadata_field_to_check_installed(provider).await?)
}
