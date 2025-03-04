// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod api;
pub mod launcher;
pub mod models;
pub mod state;
pub mod utils;

use launcher::{process_exit, setup_app};
pub use utils::result::*;

use api::{
    get_action_on_instance_launch, get_loader_versions_manifest, get_minecraft_version_manifest,
    get_progress_bars, initialize_state, load_enabled_plugins, reveal_in_explorer,
    set_action_on_instance_launch,
};

fn main() {
    tauri::Builder::default()
        .setup(move |app| Ok(setup_app(app)?))
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        // Local plugins
        .plugin(api::auth::init())
        .plugin(api::instance::init())
        .plugin(api::process::init())
        .plugin(api::plugin::init())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            load_enabled_plugins,
            get_minecraft_version_manifest,
            get_loader_versions_manifest,
            get_progress_bars,
            reveal_in_explorer,
            get_action_on_instance_launch,
            set_action_on_instance_launch
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |app, event| process_exit(app, &event));
}
