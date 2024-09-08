// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod api;
pub mod utils;

use api::tauri::{get_minecraft_version_manifest, initialize_state};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            get_minecraft_version_manifest
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
