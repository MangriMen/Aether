use std::path::PathBuf;

use tauri::api::path::{app_cache_dir, app_data_dir};

pub fn get_app_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    app_data_dir(&app_handle.config()).unwrap()
}

pub fn get_cache_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    app_cache_dir(&app_handle.config()).unwrap()
}
