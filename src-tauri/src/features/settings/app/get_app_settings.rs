use tauri::State;

use crate::{
    features::settings::{AppSettings, AppSettingsState},
    FrontendResult,
};

pub async fn get_app_settings_use_case(
    state: State<'_, AppSettingsState>,
) -> FrontendResult<AppSettings> {
    let settings_state = state.lock().await;
    Ok(*settings_state)
}
