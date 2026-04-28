use std::env;

use tauri::{Builder, Wry};

use crate::features::{auth, events, instance, minecraft, plugins, process, settings, update};
use crate::shared::specta::get_all_features_builders;

use super::{
    super::api::{self, __cmd__reveal_in_explorer, reveal_in_explorer},
    super::window::get_main_window_state_flags,
    events::handle_app_events,
    initialize::init_app,
    log::default_log_builder,
    pipe::Pipe,
};

/// Entry point for the Tauri runtime
/// Orchestrates the build process and starts the event loop
pub fn launch_app() -> crate::Result<()> {
    build_app()?.run(handle_app_events);
    Ok(())
}

fn build_app() -> crate::Result<tauri::App> {
    create_app()
        .build(tauri::generate_context!())
        .map_err(|e| crate::Error::LaunchError(e.to_string()))
}

fn create_app() -> Builder<Wry> {
    let builders_with_name = get_all_features_builders();

    #[cfg(debug_assertions)]
    crate::shared::specta::export_specta_builders(&builders_with_name);

    Builder::default()
        .setup(move |app| {
            for (_, builder) in &builders_with_name {
                builder.mount_events(app);
            }

            init_app(app);

            Ok(())
        })
        .pipe(configure_system_plugins)
        .pipe(configure_feature_plugins)
        .pipe(configure_commands)
}

fn configure_system_plugins(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(default_log_builder().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(get_main_window_state_flags())
                .build(),
        )
}

fn configure_feature_plugins(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .plugin(api::init())
        .plugin(auth::init())
        .plugin(events::init())
        .plugin(instance::init())
        .plugin(minecraft::init())
        .plugin(process::init())
        .plugin(plugins::init())
        .plugin(settings::init())
        .plugin(update::init())
}

fn configure_commands(builder: Builder<Wry>) -> Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![reveal_in_explorer,])
}
