use daedalus::{minecraft, modded};

use crate::FrontendResult;

#[tauri::command]
pub async fn get_minecraft_version_manifest() -> FrontendResult<minecraft::VersionManifest> {
    Ok(aether_core::api::metadata::get_version_manifest().await?)
}

#[tauri::command]
pub async fn get_loader_version_manifest(loader: String) -> FrontendResult<modded::Manifest> {
    Ok(aether_core::api::metadata::get_loader_version_manifest(loader).await?)
}
