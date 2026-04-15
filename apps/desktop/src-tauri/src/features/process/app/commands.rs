use aether_core::features::process::MinecraftProcessMetadata;

use crate::{
    commands::{process_commands, PROCESS_PLUGIN_NAME},
    FrontendResult,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(PROCESS_PLUGIN_NAME)
        .invoke_handler(process_commands!(tauri::generate_handler!))
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
