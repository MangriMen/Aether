use tracing::info;

use crate::{
    features::auth::{
        app::CredentialsStorage,
        infra::{FsCredentialsStorage, SqliteCredentialsStorage},
    },
    shared::archive::infra::archive_legacy_file,
};

pub async fn migrate_credentials_to_sqlite(
    fs_storage: &FsCredentialsStorage,
    sqlite_storage: &SqliteCredentialsStorage,
    migrated_dir_name: &str,
) -> crate::Result<()> {
    let old_path = fs_storage.get_file_path();

    // No need to migrate
    if !old_path.exists() {
        return Ok(());
    }

    if !sqlite_storage.list().await?.is_empty() {
        return Ok(());
    }

    if let Ok(old_credentials) = fs_storage.list().await {
        if old_credentials.is_empty() {
            return Ok(());
        }

        info!("Legacy credentials found. Starting migration to SQLite...");

        sqlite_storage.upsert_all(old_credentials).await?;

        archive_legacy_file(&old_path, migrated_dir_name, "credentials").await;
    }

    Ok(())
}
