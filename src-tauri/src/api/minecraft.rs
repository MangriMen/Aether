use aether_core::event::LoadingBar;
use dashmap::DashMap;
use uuid::Uuid;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_progress_bars() -> AetherLauncherResult<DashMap<Uuid, LoadingBar>> {
    Ok(aether_core::state::EventState::list_progress_bars().await?)
}
