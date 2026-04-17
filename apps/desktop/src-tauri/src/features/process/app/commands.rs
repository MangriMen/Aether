use crate::{
    commands::{process_commands, PROCESS_PLUGIN_NAME},
    features::process::MinecraftProcessMetadataDto,
    FrontendResult,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(PROCESS_PLUGIN_NAME)
        .invoke_handler(process_commands!(tauri::generate_handler!))
        .build()
}

pub fn get_specta_data<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    process_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
async fn list() -> FrontendResult<Vec<MinecraftProcessMetadataDto>> {
    Ok(aether_core::api::process::list()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get_by_instance_id(id: String) -> FrontendResult<Vec<MinecraftProcessMetadataDto>> {
    Ok(aether_core::api::process::get_by_instance_id(id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}
