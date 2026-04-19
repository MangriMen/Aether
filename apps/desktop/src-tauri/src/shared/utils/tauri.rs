use std::path::PathBuf;

use tauri::Manager;

#[must_use]
pub fn get_app_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle.path().app_data_dir().unwrap()
}

#[must_use]
pub fn get_cache_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle.path().app_cache_dir().unwrap()
}
