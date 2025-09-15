use tauri::{Builder, Wry};

use crate::{
    core::commands::*,
    features::{
        auth, events::commands::*, instance, minecraft::commands::*, plugins, process, settings,
    },
};

use super::{events::handle_app_events, initialize::init_app};

// Entry point for Tauri runtime and plugin setup
pub fn launch_app() -> crate::Result<()> {
    build_app()?.run(handle_app_events);
    Ok(())
}

fn build_app() -> crate::Result<tauri::App> {
    create_tauri_app()
        .build(tauri::generate_context!())
        .map_err(|e| crate::Error::LaunchError(e.to_string()))
}

fn create_tauri_app() -> Builder<Wry> {
    Builder::default()
        .setup(move |app| Ok(init_app(app)?))
        .pipe(with_tauri_plugins)
        .pipe(with_feature_plugins)
}

fn with_tauri_plugins(builder: Builder<Wry>) -> Builder<Wry> {
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
fn with_feature_plugins(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .plugin(auth::init())
        .plugin(instance::init())
        .plugin(process::init())
        .plugin(plugins::init())
        .plugin(settings::init())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            initialize_plugins,
            get_minecraft_version_manifest,
            get_loader_version_manifest,
            get_progress_bars,
            reveal_in_explorer,
        ])
}

/// Helper trait for method chaining
pub trait Pipe {
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
