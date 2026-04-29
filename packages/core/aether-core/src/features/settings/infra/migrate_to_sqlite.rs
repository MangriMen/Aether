use tracing::{info, warn};

use crate::features::settings::{
    SettingsStorage,
    infra::{FsSettingsStorage, SqliteSettingsStorage},
};

pub async fn migrate_to_sqlite(
    fs_storage: &FsSettingsStorage,
    sqlite_storage: &SqliteSettingsStorage,
) -> crate::Result<()> {
    if sqlite_storage.get().await.is_ok() {
        return Ok(());
    }

    if let Ok(old_settings) = fs_storage.get().await {
        info!("Legacy settings found. Starting migration to SQLite...");

        sqlite_storage.upsert(old_settings).await?;

        let old_path = fs_storage.get_file_path();
        let mut backup_path = old_path.clone();
        backup_path.set_extension("json.bak");

        if let Err(e) = std::fs::rename(&old_path, &backup_path) {
            warn!(
                "Failed to rename legacy settings file: {}. Migration won't run again anyway because DB is now populated.",
                e
            );
        } else {
            info!("Legacy settings moved to {:?}", backup_path);
        }
    }

    Ok(())
}
