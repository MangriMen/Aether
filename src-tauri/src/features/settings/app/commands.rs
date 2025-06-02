use aether_core::features::settings::Settings;

use crate::FrontendResult;

#[tauri::command]
pub async fn get_settings() -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::get().await?)
}

#[tauri::command]
pub async fn get_max_ram() -> FrontendResult<u64> {
    Ok(aether_core::shared::infra::get_total_memory())
}
