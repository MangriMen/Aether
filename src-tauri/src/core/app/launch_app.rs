use tauri::{Builder, Wry};

use crate::{
    core::{commands::*, handle_app_event, setup_app, utils::*},
    features::{
        app_settings::commands::*, auth, events::commands::*, instance, minecraft::commands::*,
        plugins, process, settings::commands::*,
    },
};

pub fn launch_app() {
    get_tauri_app()
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |app, event| handle_app_event(app, &event));
}

pub fn get_tauri_app() -> Builder<Wry> {
    Builder::default()
        .setup(move |app| Ok(setup_app(app)?))
        .pipe(setup_tauri_plugins)
        .pipe(setup_commands)
}

fn setup_tauri_plugins(builder: Builder<Wry>) -> Builder<Wry> {
    builder
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
}

/// Configures application-specific commands and plugins
fn setup_commands(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .plugin(auth::init())
        .plugin(instance::init())
        .plugin(process::init())
        .plugin(plugins::init())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            initialize_plugins,
            get_minecraft_version_manifest,
            get_loader_version_manifest,
            get_progress_bars,
            reveal_in_explorer,
            get_settings,
            get_max_ram,
            get_app_settings,
            update_app_settings,
        ])
}

/// Helper trait for method chaining
trait Pipe {
    fn pipe<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
        Self: Sized;
}

impl<T> Pipe for T {
    fn pipe<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}
