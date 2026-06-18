pub(crate) mod infra;

// Facade: explicit re-exports (Rule 4.3)
pub(crate) use infra::tauri::commands::get_specta_commands;
pub(crate) use infra::tauri::commands::init;
pub use infra::tauri::dtos::EditJavaDto;
pub use infra::tauri::dtos::InstallJavaDto;
pub use infra::tauri::dtos::JavaDto;
