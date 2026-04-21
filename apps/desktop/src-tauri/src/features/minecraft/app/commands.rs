use crate::{
    FrontendResult,
    commands::{MINECRAFT_PLUGIN_NAME, minecraft_commands},
    features::minecraft::{ModLoaderDto, modded, vanilla},
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
pub async fn get_minecraft_version_manifest() -> FrontendResult<vanilla::VersionManifestDto> {
    Ok(aether_core::api::metadata::get_version_manifest()
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
pub async fn get_loader_version_manifest(
    loader: ModLoaderDto,
) -> FrontendResult<modded::ModdedManifestDto> {
    Ok(
        aether_core::api::metadata::get_loader_version_manifest(loader.into())
            .await
            .map_err(crate::Error::from)?
            .into(),
    )
}
