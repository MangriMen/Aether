mod default_instance_settings;
mod error;
mod location_info;
mod settings;

pub use default_instance_settings::{
    DefaultInstanceSettings, Hooks, MemorySettings, WindowSettings, WindowSize,
};
pub use error::SettingsError;
pub use location_info::{
    CACHE_FOLDER_NAME, INSTANCES_FOLDER_NAME, LocationInfo, METADATA_FOLDER_NAME,
    PLUGINS_FOLDER_NAME,
};
pub use settings::{
    DEFAULT_MAX_CONCURRENT_DOWNLOADS, DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64, Settings,
};
