// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod api;
pub mod models;
pub mod state;
pub mod utils;

pub use utils::result::*;

use tauri::Manager;

use api::tauri::{
    call_plugin, change_account, create_minecraft_instance, create_offline_account, get_accounts,
    get_loader_versions_manifest, get_minecraft_instance_process, get_minecraft_instances,
    get_minecraft_version_manifest, get_progress_bars, get_running_minecraft_instances,
    initialize_state, launch_minecraft_instance, logout, remove_minecraft_instance,
    reveal_in_explorer, stop_minecraft_instance,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .max_file_size(50_000 /* bytes */)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .level(log::LevelFilter::Trace)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(crate::state::AppState::default());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            call_plugin,
            create_minecraft_instance,
            get_minecraft_instances,
            get_minecraft_version_manifest,
            get_loader_versions_manifest,
            get_progress_bars,
            get_running_minecraft_instances,
            initialize_state,
            get_minecraft_instance_process,
            stop_minecraft_instance,
            launch_minecraft_instance,
            remove_minecraft_instance,
            get_accounts,
            reveal_in_explorer,
            create_offline_account,
            change_account,
            logout
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
