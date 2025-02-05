// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod api;
pub mod models;
pub mod state;
pub mod utils;

use state::{load_settings, ActionOnInstanceLaunch, SettingsState};
use tokio::sync::Mutex;
pub use utils::result::*;

use tauri::{Listener, Manager, WebviewWindowBuilder};

use api::tauri::{
    call_plugin, change_account, create_minecraft_instance, create_offline_account,
    edit_minecraft_instance, get_accounts, get_action_on_instance_launch,
    get_loader_versions_manifest, get_minecraft_instance, get_minecraft_instance_process,
    get_minecraft_instances, get_minecraft_version_manifest, get_progress_bars,
    get_running_minecraft_instances, initialize_state, launch_minecraft_instance, logout,
    remove_minecraft_instance, reveal_in_explorer, set_action_on_instance_launch,
    stop_minecraft_instance,
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
                .level(log::LevelFilter::Trace)
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
                    let app_handle = app_handle.clone();
                    let payload =
                        serde_json::from_str::<aether_core::event::ProcessPayload>(e.payload());

                    tauri::async_runtime::spawn(async move {
                        async fn on_instance_launch(app_handle: tauri::AppHandle) {
                            let settings_state_handle = app_handle.state::<SettingsState>();
                            let settings_state = settings_state_handle.lock().await;

                            let action_on_launch = &settings_state.action_on_instance_launch;

                            match action_on_launch {
                                ActionOnInstanceLaunch::Hide => {
                                    log::info!("Hide launcher");
                                    for (_label, window) in app_handle.webview_windows() {
                                        window.hide().unwrap();
                                    }
                                }
                                ActionOnInstanceLaunch::Close => {
                                    log::info!("Close launcher windows");

                                    let manual_exit_flag =
                                        app_handle.state::<ManualExitFlagState>();

                                    manual_exit_flag.lock().unwrap().0 = true;
                                    for (_label, window) in app_handle.webview_windows() {
                                        window.close().unwrap();
                                    }
                                }
                                ActionOnInstanceLaunch::Nothing => {}
                            }
                        }

                        async fn on_instance_finish(app_handle: tauri::AppHandle) {
                            let settings_state_handle = app_handle.state::<SettingsState>();
                            let settings_state = settings_state_handle.lock().await;

                            let action_on_launch = &settings_state.action_on_instance_launch;

                            match action_on_launch {
                                ActionOnInstanceLaunch::Hide => {
                                    log::info!("Show launcher");
                                    for (_label, window) in app_handle.webview_windows() {
                                        window.show().unwrap();
                                    }
                                }
                                ActionOnInstanceLaunch::Close => {
                                    log::info!("Create launcher windows");
                                    for window_config in
                                        app_handle.config().app.windows.iter().filter(|w| w.create)
                                    {
                                        WebviewWindowBuilder::from_config(
                                            &app_handle,
                                            window_config,
                                        )
                                        .unwrap()
                                        .build()
                                        .unwrap();
                                    }

                                    let manual_exit_flag =
                                        app_handle.state::<ManualExitFlagState>();

                                    manual_exit_flag.lock().unwrap().0 = false;
                                }
                                ActionOnInstanceLaunch::Nothing => {}
                            }
                        }

                        match payload {
                            Ok(payload) => match payload.event {
                                aether_core::event::ProcessPayloadType::Launched => {
                                    on_instance_launch(app_handle.clone()).await;
                                }
                                aether_core::event::ProcessPayloadType::Finished => {
                                    on_instance_finish(app_handle.clone()).await;
                                }
                            },
                            Err(err) => {
                                log::error!("Failed to parse process event payload: {}", err);
                            }
                        }
                    });
                },
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            call_plugin,
            create_minecraft_instance,
            edit_minecraft_instance,
            get_minecraft_instances,
            get_minecraft_instance,
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
            logout,
            get_action_on_instance_launch,
            set_action_on_instance_launch
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |app, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                let manual_exit_flag = app.state::<ManualExitFlagState>();
                let manual_exit_flag = manual_exit_flag.lock().unwrap();
                if manual_exit_flag.0 {
                    api.prevent_exit();
                }
            }
            _ => (),
        });
}
