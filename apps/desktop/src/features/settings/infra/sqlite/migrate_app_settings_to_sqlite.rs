use aether_core::shared::archive::infra::{archive_legacy_file, quarantine_unreadable_legacy_file};
use log::info;

use crate::features::settings::domain::AppSettingsStorage;
use crate::features::settings::infra::{FsAppSettingsStorage, SqliteAppSettingsStorage};

pub async fn migrate_app_settings_to_sqlite(
    fs_storage: &FsAppSettingsStorage,
    sqlite_storage: &SqliteAppSettingsStorage,
    migrated_dir_name: &str,
) -> crate::Result<()> {
    let old_path = fs_storage.get_file_path();

    // No need to migrate
    if !old_path.exists() {
        return Ok(());
    }

    if sqlite_storage.exists().await {
        return Ok(());
    }

    let old_app_settings = match fs_storage.read_strict().await {
        Ok(value) => value,
        Err(err) => {
            quarantine_unreadable_legacy_file(&old_path, migrated_dir_name, "app settings").await;
            return Err(err.into());
        }
    };

    info!("Legacy app settings found. Starting migration to SQLite...");

    sqlite_storage.upsert(old_app_settings).await?;

    archive_legacy_file(&old_path, migrated_dir_name, "app settings").await;

    Ok(())
}
