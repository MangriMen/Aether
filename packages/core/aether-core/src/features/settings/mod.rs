mod app;
mod domain;
pub mod infra;

pub use domain::{
    CACHE_FOLDER_NAME, DEFAULT_MAX_CONCURRENT_DOWNLOADS, DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64,
    DefaultInstanceSettings, Hooks, INSTANCES_FOLDER_NAME, LocationInfo, METADATA_FOLDER_NAME,
    MemorySettings, PLUGINS_FOLDER_NAME, Settings, SettingsError, WindowSettings, WindowSize,
};

pub use app::{
    DefaultInstanceSettingsStorage, EditDefaultInstanceSettings,
    EditDefaultInstanceSettingsUseCase, EditHooks, EditSettings, EditSettingsUseCase,
    GetDefaultInstanceSettingsUseCase, GetSettingsUseCase, SettingsStorage,
};
