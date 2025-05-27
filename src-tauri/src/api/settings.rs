use aether_core::features::settings::Settings;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_settings() -> AetherLauncherResult<Settings> {
    Ok(aether_core::api::settings::get().await?)
}

#[tauri::command]
pub async fn get_max_ram() -> AetherLauncherResult<u64> {
    Ok(aether_core::shared::infra::get_total_memory())
}
