use std::fs;

use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ActionOnInstanceLaunch {
    #[default]
    Nothing,
    Hide,
    Close,
}

#[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct SettingsStateInner {
    pub action_on_instance_launch: ActionOnInstanceLaunch,
}

pub type SettingsState = Mutex<SettingsStateInner>;

fn get_settings_path(app_handle: &AppHandle) -> std::path::PathBuf {
    app_handle
        .path()
        .app_config_dir()
        .unwrap()
        .join("aether_settings.json")
}

pub fn load_settings(app_handle: AppHandle) -> SettingsStateInner {
    let path = get_settings_path(&app_handle);

    if let Ok(contents) = fs::read_to_string(&path) {
        serde_json::from_str(&contents).unwrap_or_else(|_| {
            let app_handle = app_handle.clone();
            let settings = SettingsStateInner::default();

            tauri::async_runtime::spawn(async move {
                save_settings(app_handle, &settings).await;
            });

            settings
        })
    } else {
        SettingsStateInner::default()
    }
}

pub async fn save_settings(app_handle: AppHandle, state: &SettingsStateInner) {
    let path = get_settings_path(&app_handle);

    tokio::fs::write(&path, serde_json::to_string(state).unwrap())
        .await
        .unwrap();
}
