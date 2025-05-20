use aether_core::{
    core::domain::LazyLocator,
    features::events::{ProgressBar, ProgressBarStorage},
};
use dashmap::DashMap;
use uuid::Uuid;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_progress_bars() -> AetherLauncherResult<DashMap<Uuid, ProgressBar>> {
    let lazy_locator = LazyLocator::get().await?;

    let progress_bar_storage = lazy_locator.get_progress_bar_storage().await;

    Ok(progress_bar_storage.list().await)
}
