pub(crate) mod infra;

// Facade: explicit re-exports (Rule 4.3)
pub(crate) use infra::tauri::commands::get_specta_commands;
pub use infra::tauri::commands::get_specta_events;
pub(crate) use infra::tauri::commands::init;
pub use infra::tauri::dtos::MinecraftProcessMetadataDto;
pub use infra::tauri::dtos::ProcessEventDto;
