mod models;

pub(crate) use models::parse_platform_providers;
pub use models::{
    CACHE_FOLDER_NAME, DefaultInstanceSettings, Hooks, INSTANCE_METADATA_FOLDER_NAME,
    INSTANCES_FOLDER_NAME, InvalidMaxConcurrentDownloads, InvalidMemoryMb, LocationInfo,
    METADATA_FOLDER_NAME, MaxConcurrentDownloads, MemoryMb, MemorySettings,
    PLUGIN_SETTINGS_FILE_NAME, PLUGINS_FOLDER_NAME, Settings, SettingsError, WindowSettings,
    WindowSize,
};
