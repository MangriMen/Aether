// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod api;
pub mod models;
pub mod utils;

use api::tauri::{
    create_minecraft_instance, get_minecraft_instances, get_minecraft_version_manifest,
    initialize_state, launch_minecraft_instance,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            get_minecraft_version_manifest,
            create_minecraft_instance,
            get_minecraft_instances,
            launch_minecraft_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
