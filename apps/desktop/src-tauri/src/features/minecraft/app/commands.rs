use crate::{
    commands::{minecraft_commands, MINECRAFT_PLUGIN_NAME},
    features::minecraft::{mod_loader::ModLoaderDto, modded, vanilla},
    FrontendResult,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(MINECRAFT_PLUGIN_NAME)
        .invoke_handler(minecraft_commands!(tauri::generate_handler!))
        .build()
}

#[tauri::command]
pub async fn get_minecraft_version_manifest() -> FrontendResult<vanilla::VersionManifestDto> {
    Ok(aether_core::api::metadata::get_version_manifest()
        .await?
        .into())
}

#[tauri::command]
pub async fn get_loader_version_manifest(
    loader: ModLoaderDto,
) -> FrontendResult<modded::ManifestDto> {
    Ok(
        aether_core::api::metadata::get_loader_version_manifest(loader.into())
            .await?
            .into(),
    )
}
