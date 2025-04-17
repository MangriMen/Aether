use daedalus::{minecraft, modded};

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_minecraft_version_manifest() -> AetherLauncherResult<minecraft::VersionManifest> {
    Ok(aether_core::api::metadata::get_version_manifest().await?)
}

#[tauri::command]
pub async fn get_loader_version_manifest(loader: String) -> AetherLauncherResult<modded::Manifest> {
    Ok(aether_core::api::metadata::get_loader_version_manifest(&loader).await?)
}
