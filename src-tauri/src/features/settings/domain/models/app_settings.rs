use std::fs;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionOnInstanceLaunch {
    #[default]
    Nothing,
    Hide,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowEffect {
    #[default]
    Off,
    MicaLight,
    MicaDark,
    Mica,
    Acrylic,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub action_on_instance_launch: ActionOnInstanceLaunch,
    pub transparent: bool,
    pub window_effect: WindowEffect,
}

pub type AppSettingsState = Mutex<AppSettings>;

fn get_settings_path<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> std::path::PathBuf {
    app_handle
        .path()
        .app_config_dir()
        .unwrap()
        .join("aether_settings.json")
}

pub fn load_settings<R: tauri::Runtime>(app_handle: AppHandle<R>) -> AppSettings {
    let path = get_settings_path(&app_handle);

    if let Ok(contents) = fs::read_to_string(&path) {
        serde_json::from_str(&contents).unwrap_or_else(|_| {
            let app_handle = app_handle.clone();
            let settings = AppSettings::default();

            tauri::async_runtime::spawn(async move {
                save_settings(app_handle, &settings).await;
            });

            settings
        })
    } else {
        AppSettings::default()
    }
}

pub async fn save_settings<R: tauri::Runtime>(app_handle: AppHandle<R>, state: &AppSettings) {
    let path = get_settings_path(&app_handle);

    tokio::fs::write(&path, serde_json::to_string(state).unwrap())
        .await
        .unwrap();
}
