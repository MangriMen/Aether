use std::path::PathBuf;

use aether_core::shared::io::infra::create_dir_all;

pub async fn create_pool(db_path: PathBuf) -> crate::Result<sqlx::SqlitePool> {
    if let Some(parent) = db_path.parent() {
        create_dir_all(parent)
            .await
            .map_err(|err| crate::Error::LaunchError(err.to_string()))?;
    }

    let connection_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = sqlx::SqlitePool::connect_with(connection_options)
        .await
        .map_err(|err| crate::Error::LaunchError(err.to_string()))?;

    Ok(pool)
}
