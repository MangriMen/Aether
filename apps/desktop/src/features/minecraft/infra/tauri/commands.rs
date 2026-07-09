use aether_core::{core::app::AetherContainer, features::minecraft::MinecraftFeature};

use crate::{
    FrontendResult,
    features::minecraft::{ModLoaderDto, ModdedManifestDto, VersionManifestDto},
    shared::commands::{MINECRAFT_PLUGIN_NAME, minecraft_commands},
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(MINECRAFT_PLUGIN_NAME)
        .invoke_handler(minecraft_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    minecraft_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
pub async fn get_minecraft_version_manifest() -> FrontendResult<VersionManifestDto> {
    let container = AetherContainer::get();
    Ok(container
        .get_version_manifest_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
pub async fn get_loader_version_manifest(
    loader: ModLoaderDto,
) -> FrontendResult<ModdedManifestDto> {
    let container = AetherContainer::get();
    Ok(container
        .get_loader_version_manifest_use_case()
        .execute(loader.into())
        .await
        .map_err(aether_core::Error::from)?
        .into())
}
