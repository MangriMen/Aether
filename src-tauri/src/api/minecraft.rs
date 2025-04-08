use aether_core::features::events::{EventState, LoadingBar};
use dashmap::DashMap;
use uuid::Uuid;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_progress_bars() -> AetherLauncherResult<DashMap<Uuid, LoadingBar>> {
    Ok(EventState::list_progress_bars().await?)
}
