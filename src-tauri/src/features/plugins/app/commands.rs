use std::path::PathBuf;

use aether_core::core::LauncherState;
use aether_core::features::plugins::{
    CapabilityEntry, EditPluginSettings, ImporterCapability, PluginDto, PluginSettings,
    UpdaterCapability,
};

use crate::shared::file::reveal_in_explorer;
use crate::FrontendResult;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("plugin")
        .invoke_handler(tauri::generate_handler![
            import,
            sync,
            list,
            get,
            remove,
            enable,
            disable,
            call,
            get_settings,
            edit_settings,
            open_plugins_folder,
            list_importers,
            list_updaters,
            get_api_version
        ])
        .build()
}

#[tauri::command]
async fn import(paths: Vec<PathBuf>) -> FrontendResult<()> {
    Ok(aether_core::api::plugin::import(paths).await?)
}

#[tauri::command]
async fn sync() -> FrontendResult<()> {
    Ok(aether_core::api::plugin::sync().await?)
}

#[tauri::command]
async fn list() -> FrontendResult<Vec<PluginDto>> {
    Ok(aether_core::api::plugin::list().await?)
}

#[tauri::command]
async fn get(id: String) -> FrontendResult<PluginDto> {
    Ok(aether_core::api::plugin::get(id).await?)
}

#[tauri::command]
async fn remove(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::plugin::remove(id).await?)
}

#[tauri::command]
async fn enable(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::plugin::enable(id).await?)
}

#[tauri::command]
async fn disable(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::plugin::disable(id).await?)
}

#[tauri::command]
async fn call(id: String, data: String) -> FrontendResult<()> {
    Ok(aether_core::api::plugin::call(id, data).await?)
}

#[tauri::command]
async fn get_settings(id: String) -> FrontendResult<Option<PluginSettings>> {
    Ok(aether_core::api::plugin::get_settings(id).await?)
}

#[tauri::command]
async fn edit_settings(id: String, edit_settings: EditPluginSettings) -> FrontendResult<()> {
    Ok(aether_core::api::plugin::edit_settings(id, edit_settings).await?)
}

#[tauri::command]
async fn open_plugins_folder() -> FrontendResult<()> {
    let state = LauncherState::get().await?;
    Ok(reveal_in_explorer(
        &state.location_info.plugins_dir(),
        true,
    )?)
}

#[tauri::command]
async fn list_importers() -> FrontendResult<Vec<CapabilityEntry<ImporterCapability>>> {
    Ok(aether_core::api::plugin::list_importers().await?)
}

#[tauri::command]
async fn list_updaters() -> FrontendResult<Vec<CapabilityEntry<UpdaterCapability>>> {
    // Ok(aether_core::api::plugin::list_importers().await?)
    todo!()
}

#[tauri::command]
async fn get_api_version() -> FrontendResult<semver::Version> {
    Ok(aether_core::api::plugin::get_api_version().await?)
}
