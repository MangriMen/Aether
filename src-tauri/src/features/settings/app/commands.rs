use aether_core::features::settings::{
    DefaultInstanceSettings, EditDefaultInstanceSettings, Settings,
};

use crate::FrontendResult;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("settings")
        .invoke_handler(tauri::generate_handler![
            get,
            edit,
            get_max_ram,
            get_default_instance_settings,
            edit_default_instance_settings
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
pub async fn get_default_instance_settings() -> FrontendResult<DefaultInstanceSettings> {
    Ok(aether_core::api::settings::get_default_instance_settings().await?)
}

#[tauri::command]
pub async fn edit_default_instance_settings(
    edit_settings: EditDefaultInstanceSettings,
) -> FrontendResult<DefaultInstanceSettings> {
    Ok(aether_core::api::settings::upsert_default_instance_settings(edit_settings).await?)
}
