use aether_core::{core::domain::LazyLocator, features::events::ListProgressBarsUseCase};

use crate::{features::events::ProgressBarDto, FrontendResult};

#[tauri::command]
pub async fn list_progress_bars() -> FrontendResult<Vec<ProgressBarDto>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListProgressBarsUseCase::new(lazy_locator.get_progress_bar_storage().await)
            .execute()
            .await
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}
