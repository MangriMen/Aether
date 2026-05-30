mod fs;
mod sqlite;

pub use fs::{
    FsDefaultInstanceSettingsStorage, FsSettingsStorage,
    models::{HooksV2, MemorySettingsV1, WindowSettingsV1, WindowSizeV1},
};
pub use sqlite::{
    SqliteDefaultInstanceSettingsStorage, SqliteSettingsStorage,
    migrate_default_instance_settings_to_sqlite, migrate_settings_to_sqlite,
};
