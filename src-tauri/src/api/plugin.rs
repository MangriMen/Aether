use aether_core::core::LauncherState;
use aether_core::features::plugins::{PluginMetadata, PluginSettings};

use crate::AetherLauncherResult;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("plugin")
        .invoke_handler(tauri::generate_handler![
            scan_plugins,
            list_plugins,
            plugin_get,
            is_plugin_enabled,
            enable_plugin,
            disable_plugin,
            call_plugin,
            plugin_get_settings,
            plugin_edit_settings,
            open_plugins_folder,
        ])
        .build()
}

#[tauri::command]
pub async fn scan_plugins() -> AetherLauncherResult<()> {
    Ok(aether_core::api::plugin::scan().await?)
}

#[tauri::command]
pub async fn list_plugins() -> AetherLauncherResult<Vec<PluginMetadata>> {
    Ok(aether_core::api::plugin::list().await?)
}

#[tauri::command]
pub async fn plugin_get(id: String) -> AetherLauncherResult<PluginMetadata> {
    Ok(aether_core::api::plugin::get(&id).await?)
}

#[tauri::command]
pub async fn is_plugin_enabled(id: String) -> AetherLauncherResult<bool> {
    Ok(aether_core::api::plugin::is_enabled(&id).await?)
}

#[tauri::command]
pub async fn enable_plugin(id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::plugin::enable(&id).await?)
}

#[tauri::command]
pub async fn disable_plugin(id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::plugin::disable(&id).await?)
}

#[tauri::command]
pub async fn call_plugin(id: String, data: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::plugin::call(&id, &data).await?)
}

#[tauri::command]
pub async fn plugin_get_settings(id: String) -> AetherLauncherResult<PluginSettings> {
    Ok(aether_core::api::plugin::get_settings(&id).await?)
}

#[tauri::command]
pub async fn plugin_edit_settings(
    id: String,
    settings: PluginSettings,
) -> AetherLauncherResult<()> {
    Ok(aether_core::api::plugin::edit_settings(&id, &settings).await?)
}

#[tauri::command]
pub async fn open_plugins_folder() -> AetherLauncherResult<()> {
    let state = LauncherState::get().await?;
    crate::utils::file::reveal_in_explorer(&state.locations.plugins_dir(), true)
}
