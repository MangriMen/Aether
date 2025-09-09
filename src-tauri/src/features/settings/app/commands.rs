use aether_core::features::settings::{
    EditGlobalInstanceSettings, GlobalInstanceSettings, Settings,
};

use crate::FrontendResult;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("settings")
        .invoke_handler(tauri::generate_handler![
            get,
            edit,
            get_max_ram,
            get_global_instance_settings,
            edit_global_instance_settings
        ])
        .build()
}

#[tauri::command]
pub async fn get() -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::get().await?)
}

#[tauri::command]
pub async fn edit(edit_settings: Settings) -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::upsert(edit_settings).await?)
}

#[tauri::command]
pub async fn get_max_ram() -> FrontendResult<u64> {
    Ok(aether_core::shared::infra::get_total_memory())
}

#[tauri::command]
pub async fn get_global_instance_settings() -> FrontendResult<GlobalInstanceSettings> {
    Ok(aether_core::api::settings::get_global_instance_settings().await?)
}

#[tauri::command]
pub async fn edit_global_instance_settings(
    edit_settings: EditGlobalInstanceSettings,
) -> FrontendResult<GlobalInstanceSettings> {
    Ok(aether_core::api::settings::upsert_global_instance_settings(edit_settings).await?)
}
