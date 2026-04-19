use std::env;

use log::LevelFilter;

const DEFAULT_LOG_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
const DEFAULT_LOG_COUNT: usize = 5;

#[must_use]
pub fn default_log_builder() -> tauri_plugin_log::Builder {
    let log_builder = tauri_plugin_log::Builder::new();

    let rust_log_env = env::var("RUST_LOG");

    if rust_log_env.is_ok_and(|v| !v.is_empty()) {
        log_builder
    } else {
        let (default_level, external_level) = if cfg!(debug_assertions) {
            (LevelFilter::Debug, LevelFilter::Info)
        } else {
            (LevelFilter::Info, LevelFilter::Warn)
        };

        log_builder
            .max_file_size(DEFAULT_LOG_MAX_FILE_SIZE.into())
            .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(
                DEFAULT_LOG_COUNT,
            ))
            .level(default_level)
            // Core lib
            .level_for("aether_core", default_level)
            // Platform
            .level_for("tao", LevelFilter::Off)
            .level_for("wry", LevelFilter::Off)
            // Network stack
            .level_for("reqwest", external_level)
            .level_for("rustls", external_level)
            .level_for("h2", external_level)
            .level_for("hyper", external_level)
            .level_for("hyper_util", external_level)
            // Plugin runtime
            .level_for("wasmtime", external_level)
            .level_for("cranelift_codegen", external_level)
            // Tauri plugins
            .level_for("tauri_plugin_updater", LevelFilter::Info)
    }
}
