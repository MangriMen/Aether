use aether_core::state::{PluginMetadata, PluginSettings};

use crate::AetherLauncherResult;

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
