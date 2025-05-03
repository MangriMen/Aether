use aether_core::features::events::{EventState, ProgressBar};
use dashmap::DashMap;
use uuid::Uuid;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_progress_bars() -> AetherLauncherResult<DashMap<Uuid, ProgressBar>> {
    Ok(EventState::list_progress_bars().await?)
}
