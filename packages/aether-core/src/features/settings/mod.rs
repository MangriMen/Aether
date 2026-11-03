mod app;
mod domain;
pub mod infra;

pub(crate) use domain::parse_platform_providers;
pub use domain::{
    CACHE_FOLDER_NAME, DefaultInstanceSettings, Hooks, INSTANCE_METADATA_FOLDER_NAME,
    INSTANCES_FOLDER_NAME, InvalidMaxConcurrentDownloads, InvalidMemoryMb, LocationInfo,
    METADATA_FOLDER_NAME, MaxConcurrentDownloads, MemoryMb, MemorySettings,
    PLUGIN_SETTINGS_FILE_NAME, PLUGINS_FOLDER_NAME, Settings, SettingsError, WindowSettings,
    WindowSize,
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
