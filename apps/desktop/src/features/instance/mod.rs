pub(crate) mod infra;

// Facade re-exports from infra layer (Rule 4.3)
pub(crate) use infra::tauri::api::commands::get_specta_commands;
pub(crate) use infra::tauri::api::commands::get_specta_events;
pub(crate) use infra::tauri::api::commands::init;
