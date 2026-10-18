mod app;
mod domain;
pub mod infra;

pub use domain::{
    CACHE_FOLDER_NAME, DEFAULT_MAX_CONCURRENT_DOWNLOADS, DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64,
    DefaultInstanceSettings, Hooks, INSTANCE_METADATA_FOLDER_NAME, INSTANCES_FOLDER_NAME,
    LocationInfo, METADATA_FOLDER_NAME, MemorySettings, PLUGIN_SETTINGS_FILE_NAME,
    PLUGINS_FOLDER_NAME, Settings, SettingsError, WindowSettings, WindowSize,
};

pub use app::{
    CacheKind, CacheStorageUseCase, CacheUsage, CacheUsageEntry, ClearCacheOutcome,
    ClearCacheUseCasePort, DefaultInstanceSettingsStorage, EditDefaultInstanceSettings,
    EditDefaultInstanceSettingsUseCase, EditDefaultInstanceSettingsUseCasePort, EditHooks,
    EditPlatformProvider, EditSettings, EditSettingsUseCase, EditSettingsUseCasePort,
    GetCacheUsageUseCasePort, GetDefaultInstanceSettingsUseCase,
    GetDefaultInstanceSettingsUseCasePort, GetSettingsUseCase, GetSettingsUseCasePort,
    GetStorageUsageUseCase, GetStorageUsageUseCasePort, IsCacheBusyUseCasePort, SettingsFeature,
    SettingsStorage, StorageKind, StorageUsage, StorageUsageEntry,
};
