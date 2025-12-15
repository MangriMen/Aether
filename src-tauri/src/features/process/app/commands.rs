use aether_core::features::process::MinecraftProcessMetadata;

use crate::FrontendResult;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("process")
        .invoke_handler(tauri::generate_handler![list, get_by_instance_id])
        .build()
}

#[tauri::command]
async fn list() -> FrontendResult<Vec<MinecraftProcessMetadata>> {
    Ok(aether_core::api::process::list().await?)
}

#[tauri::command]
async fn get_by_instance_id(id: String) -> FrontendResult<Vec<MinecraftProcessMetadata>> {
    Ok(aether_core::api::process::get_by_instance_id(id).await?)
}
