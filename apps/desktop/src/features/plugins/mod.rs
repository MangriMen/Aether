pub(crate) mod infra;

// Facade: explicit re-exports (Rule 4.3)
pub(crate) use infra::tauri::commands::get_specta_commands;
pub use infra::tauri::commands::get_specta_events;
pub(crate) use infra::tauri::commands::init;
pub use infra::tauri::dtos::ApiConfigDto;
pub use infra::tauri::dtos::EditPluginSettingsDto;
pub use infra::tauri::dtos::GitHubPluginPreviewDto;
pub use infra::tauri::dtos::GitHubReleaseInfoDto;
pub use infra::tauri::dtos::LoadConfigDto;
pub use infra::tauri::dtos::LoadConfigTypeDto;
pub use infra::tauri::dtos::PathMappingDto;
pub use infra::tauri::dtos::PluginCapabilitiesDto;
pub use infra::tauri::dtos::PluginContentProviderCapabilityDto;
pub use infra::tauri::dtos::PluginDto;
pub use infra::tauri::dtos::PluginEventDto;
pub use infra::tauri::dtos::PluginImporterCapabilityDto;
pub use infra::tauri::dtos::PluginManifestDto;
pub use infra::tauri::dtos::PluginMetadataDto;
pub use infra::tauri::dtos::PluginSettingsDto;
pub use infra::tauri::dtos::PluginSourceDto;
pub use infra::tauri::dtos::PluginSourceTypeDto;
pub use infra::tauri::dtos::PluginUpdateInfoDto;
pub use infra::tauri::dtos::PluginUpdaterCapabilityDto;
pub use infra::tauri::dtos::ProviderHandlersDto;
pub use infra::tauri::dtos::ProviderPluginPreviewDto;
pub use infra::tauri::dtos::RuntimeConfigDto;
