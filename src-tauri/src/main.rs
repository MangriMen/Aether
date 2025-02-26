// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod api;
pub mod launcher;
pub mod models;
pub mod state;
pub mod utils;

use state::load_settings;
use tokio::sync::Mutex;
pub use utils::result::*;

use tauri::{Listener, Manager};

use api::{
    call_plugin, change_account, create_offline_account, disable_plugin, enable_plugin,
    get_accounts, get_action_on_instance_launch, get_loader_versions_manifest,
    get_minecraft_version_manifest, get_progress_bars, initialize_state, is_plugin_enabled,
    list_plugins, logout, plugin_edit_settings, plugin_get_settings, process_get_by_instance_id,
    process_list, reveal_in_explorer, scan_plugins, set_action_on_instance_launch,
};

struct ManualExitFlagStateInner(bool);
type ManualExitFlagState = std::sync::Mutex<ManualExitFlagStateInner>;

fn main() {
    tauri::Builder::default()
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
        .setup(move |app| {
            app.manage(crate::state::AppState::default());
            app.manage(Mutex::new(load_settings(app.handle().clone())));
            app.manage(std::sync::Mutex::new(ManualExitFlagStateInner(false)));

            let app_handle = app.handle().clone();

            app.listen(
                aether_core::event::LauncherEvent::Process.as_str(),
                move |e| {
                    launcher::handle_instance_launch(&app_handle, e);
                },
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            call_plugin,
            get_minecraft_version_manifest,
            get_loader_versions_manifest,
            get_progress_bars,
            list_plugins,
            enable_plugin,
            scan_plugins,
            disable_plugin,
            is_plugin_enabled,
            process_list,
            initialize_state,
            plugin_get_settings,
            plugin_edit_settings,
            process_get_by_instance_id,
            get_accounts,
            reveal_in_explorer,
            create_offline_account,
            change_account,
            logout,
            get_action_on_instance_launch,
            set_action_on_instance_launch
        ])
        .plugin(api::instance::init())
        .plugin(api::process::init())
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let manual_exit_flag = app.state::<ManualExitFlagState>();
                let manual_exit_flag = manual_exit_flag.lock().unwrap();
                if manual_exit_flag.0 {
                    api.prevent_exit();
                }
            }
        });
}
