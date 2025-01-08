use aether_core::{event::LoadingBar, state::LauncherState};
use daedalus::minecraft;
use dashmap::DashMap;
use uuid::Uuid;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_minecraft_version_manifest() -> AetherLauncherResult<minecraft::VersionManifest> {
    let state = LauncherState::get().await?;
    let manifest = aether_core::launcher::download_version_manifest(&state, false).await?;

    Ok(manifest)
}

#[tauri::command]
pub async fn get_progress_bars() -> AetherLauncherResult<DashMap<Uuid, LoadingBar>> {
    Ok(aether_core::event::EventState::list_progress_bars().await?)
}
