use crate::{
    commands::{minecraft_commands, MINECRAFT_PLUGIN_NAME},
    features::minecraft::{modded, vanilla, ModLoaderDto},
    FrontendResult,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(MINECRAFT_PLUGIN_NAME)
        .invoke_handler(minecraft_commands!(tauri::generate_handler!))
        .build()
}

pub fn get_specta_data<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    minecraft_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
pub async fn get_minecraft_version_manifest() -> FrontendResult<vanilla::VersionManifestDto> {
    Ok(aether_core::api::metadata::get_version_manifest()
        .await?
        .into())
}

#[tauri::command]
#[specta::specta]
pub async fn get_loader_version_manifest(
    loader: ModLoaderDto,
) -> FrontendResult<modded::ModdedManifestDto> {
    Ok(
        aether_core::api::metadata::get_loader_version_manifest(loader.into())
            .await?
            .into(),
    )
}
