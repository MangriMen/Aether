mod content_providers;
mod event_emitting_instance_storage;
mod fs;
mod fs_content_icon_cache;
mod instance_event_handler;
mod legacy_provider_id;
mod services;
mod sqlite;
mod universal_content_metadata_cache;

pub use content_providers::{ModrinthContentProvider, MrpackCacheCleaner, mrpack_dir};
pub use event_emitting_instance_storage::EventEmittingInstanceStorage;
pub use fs::{FsInstanceStorage, FsPackStorage};
pub use fs_content_icon_cache::{CONTENT_ICONS_FOLDER, FsContentIconCache};
pub use instance_event_handler::InstanceEventHandler;
pub use services::{
    FsContentFileService, FsInstanceFileService, InstanceWatcherServiceImpl,
    NoPlatformProviderSelection, SettingsPlatformProviderSelection, check_content_path,
    check_plugin_write_allowed, check_size, resolve_instance_file_path, validate_relative_path,
};
pub use sqlite::{
    SqliteInstanceStorage, SqlitePackStorage, migrate_instances_to_sqlite, migrate_packs_to_sqlite,
};
pub use universal_content_metadata_cache::NAMESPACES as CONTENT_METADATA_NAMESPACES;
pub use universal_content_metadata_cache::UniversalContentMetadataCache;
