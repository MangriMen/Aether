use aether_core::core::migrations;
use sqlx::{SqlitePool, migrate::MigrateError};
use tauri::AppHandle;

use super::state;

#[derive(Debug, thiserror::Error)]
pub enum StartupMigrationError {
    #[error(transparent)]
    Migrate(#[from] MigrateError),

    #[error("Failed to prepare the database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Failed to move the old settings into the database: {0}")]
    State(crate::Error),
}

/// Migrate the legacy shared `_sqlx_migrations` table to the new split layout.
///
/// Old databases (from sqlx ≤0.8) have a single `_sqlx_migrations` table
/// containing entries for both desktop and aether-core migrations.
/// This function moves the core entries into `_sqlx_migrations_aether_core`
/// so that the new independent migrators can coexist peacefully.
///
/// Safe to call on fresh databases (no-op if the old table doesn't exist).
///
/// The split runs in one transaction (SQLite DDL is transactional), so a failure
/// halfway leaves the old table untouched instead of a half-moved state.
async fn migrate_legacy_migrations_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 1. Does the old shared table even exist?
    let (old_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations')",
    )
    .fetch_one(pool)
    .await?;

    if !old_exists {
        return Ok(());
    }

    // 2. Has the split already been done?
    let (new_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations_aether_core')",
    )
    .fetch_one(pool)
    .await?;

    if new_exists {
        // Split was done before, but the old table may still have stale
        // core entries from a previous interrupted attempt. Clean them up.
        sqlx::query(
            "DELETE FROM _sqlx_migrations
             WHERE version >= 20260505171710",
        )
        .execute(pool)
        .await?;
        return Ok(());
    }

    // 3. Perform the split: rename the old table out of the way, then create
    //    fresh tables with only the entries that belong to each package.
    //
    //    Core migrations all have version >= 20260505171710, while the sole
    //    desktop migration (`app_settings`) has version 20260504203107.
    log::info!("Detected legacy shared migration table. Splitting aether-core entries…");

    let mut tx = pool.begin().await?;

    // Rename old table so we can safely query it while sqlx creates fresh ones
    sqlx::query("ALTER TABLE _sqlx_migrations RENAME TO _sqlx_migrations_legacy_backup")
        .execute(&mut *tx)
        .await?;

    // Create the new core table populated with the core entries
    sqlx::query(
        "CREATE TABLE _sqlx_migrations_aether_core (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time INTEGER NOT NULL
        )",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO _sqlx_migrations_aether_core (version, description, installed_on, success, checksum, execution_time)
         SELECT version, description, installed_on, success, checksum, execution_time
         FROM _sqlx_migrations_legacy_backup
         WHERE version >= 20260505171710",
    )
    .execute(&mut *tx)
    .await?;

    // Create the new desktop table with only the desktop entry
    sqlx::query(
        "CREATE TABLE _sqlx_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time INTEGER NOT NULL
        )",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
         SELECT version, description, installed_on, success, checksum, execution_time
         FROM _sqlx_migrations_legacy_backup
         WHERE version < 20260505171710",
    )
    .execute(&mut *tx)
    .await?;

    // Drop the backup
    sqlx::query("DROP TABLE _sqlx_migrations_legacy_backup")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    log::info!("Successfully split legacy migration table");
    Ok(())
}

pub async fn run_migrations<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    pool: SqlitePool,
) -> Result<(), StartupMigrationError> {
    // ── Legacy migration split (one-time for existing databases) ──
    migrate_legacy_migrations_table(&pool).await?;

    // Run aether-core (library) migrations with custom table `_sqlx_migrations_aether_core`
    migrations::run_migrations(&pool).await?;

    // Run desktop (application) migrations in the default `_sqlx_migrations` table
    sqlx::migrate!("./migrations").run(&pool).await?;

    state::migrate(app_handle, pool)
        .await
        .map_err(StartupMigrationError::State)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    const CORE_VERSION: i64 = 20_260_505_171_710;
    const DESKTOP_VERSION: i64 = 20_260_504_203_107;

    const CREATE_TABLE: &str = "CREATE TABLE _sqlx_migrations (
        version INTEGER PRIMARY KEY,
        description TEXT NOT NULL,
        installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        success BOOLEAN NOT NULL,
        checksum BLOB NOT NULL,
        execution_time INTEGER NOT NULL
    )";

    async fn legacy_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(CREATE_TABLE).execute(&pool).await.unwrap();

        for (version, description) in [(DESKTOP_VERSION, "app_settings"), (CORE_VERSION, "core")] {
            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
                 VALUES (?, ?, 1, x'00', 0)",
            )
            .bind(version)
            .bind(description)
            .execute(&pool)
            .await
            .unwrap();
        }

        pool
    }

    async fn versions(pool: &SqlitePool, table: &str) -> Vec<i64> {
        let query = match table {
            "core" => "SELECT version FROM _sqlx_migrations_aether_core",
            _ => "SELECT version FROM _sqlx_migrations",
        };

        sqlx::query_scalar(query).fetch_all(pool).await.unwrap()
    }

    #[tokio::test]
    async fn should_split_legacy_table_into_desktop_and_core() {
        let pool = legacy_pool().await;

        migrate_legacy_migrations_table(&pool).await.unwrap();

        assert_eq!(versions(&pool, "core").await, vec![CORE_VERSION]);
        assert_eq!(versions(&pool, "desktop").await, vec![DESKTOP_VERSION]);
    }

    #[tokio::test]
    async fn should_keep_legacy_table_when_split_fails_midway() {
        let pool = legacy_pool().await;
        // A non-numeric version cannot go into the new INTEGER PRIMARY KEY: the copy fails after
        // the old table was renamed and the new one created, so all of that must roll back.
        sqlx::query("DROP TABLE _sqlx_migrations")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE _sqlx_migrations (
                version TEXT,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
             VALUES ('not-a-number', 'broken', 1, x'00', 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let result = migrate_legacy_migrations_table(&pool).await;

        assert!(result.is_err());
        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table'")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(tables, vec!["_sqlx_migrations".to_owned()]);
        let kept: Vec<String> = sqlx::query_scalar("SELECT version FROM _sqlx_migrations")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(kept, vec!["not-a-number".to_owned()]);
    }
}
