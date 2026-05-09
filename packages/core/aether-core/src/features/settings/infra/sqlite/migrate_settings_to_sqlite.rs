use tracing::info;

use crate::{
    features::settings::{
        SettingsStorage,
        infra::{FsSettingsStorage, SqliteSettingsStorage},
    },
    shared::archive_legacy_file,
};

pub async fn migrate_settings_to_sqlite(
    fs_storage: &FsSettingsStorage,
    sqlite_storage: &SqliteSettingsStorage,
    migrated_dir_name: &str,
) -> crate::Result<()> {
    let old_path = fs_storage.get_file_path();

    // No need to migrate
    if !old_path.exists() {
        return Ok(());
    }

    if sqlite_storage.get().await.is_ok() {
        return Ok(());
    }

    if let Ok(old_settings) = fs_storage.get().await {
        info!("Legacy settings found. Starting migration to SQLite...");

        sqlite_storage.upsert(old_settings).await?;

        archive_legacy_file(&old_path, migrated_dir_name, "settings").await;
    }

    Ok(())
}
