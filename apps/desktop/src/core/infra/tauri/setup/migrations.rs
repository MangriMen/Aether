use aether_core::core::migrations;
use sqlx::SqlitePool;
use tauri::AppHandle;

use super::state;

/// Migrate the legacy shared `_sqlx_migrations` table to the new split layout.
///
/// Old databases (from sqlx ≤0.8) have a single `_sqlx_migrations` table
/// containing entries for both desktop and aether-core migrations.
/// This function moves the core entries into `_sqlx_migrations_aether_core`
/// so that the new independent migrators can coexist peacefully.
///
/// Safe to call on fresh databases (no-op if the old table doesn't exist).
async fn migrate_legacy_migrations_table(pool: &SqlitePool) {
    // 1. Does the old shared table even exist?
    let (old_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations')",
    )
    .fetch_one(pool)
    .await
    .expect("Failed to check legacy migrations table");

    if !old_exists {
        return;
    }

    // 2. Has the split already been done?
    let (new_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations_aether_core')",
    )
    .fetch_one(pool)
    .await
    .expect("Failed to check new migrations table");

    if new_exists {
        // Split was done before, but the old table may still have stale
        // core entries from a previous interrupted attempt. Clean them up.
        sqlx::query(
            "DELETE FROM _sqlx_migrations
             WHERE version >= 20260505171710",
        )
        .execute(pool)
        .await
        .expect("Failed to clean stale core entries from legacy table");
        return;
    }

    // 3. Perform the split: rename the old table out of the way, then create
    //    fresh tables with only the entries that belong to each package.
    //
    //    Core migrations all have version >= 20260505171710, while the sole
    //    desktop migration (`app_settings`) has version 20260504203107.
    log::info!("Detected legacy shared migration table. Splitting aether-core entries…");

    // Rename old table so we can safely query it while sqlx creates fresh ones
    sqlx::query("ALTER TABLE _sqlx_migrations RENAME TO _sqlx_migrations_legacy_backup")
        .execute(pool)
        .await
        .expect("Failed to rename legacy migrations table");

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
    .execute(pool)
    .await
    .expect("Failed to create core migrations table");

    sqlx::query(
        "INSERT INTO _sqlx_migrations_aether_core (version, description, installed_on, success, checksum, execution_time)
         SELECT version, description, installed_on, success, checksum, execution_time
         FROM _sqlx_migrations_legacy_backup
         WHERE version >= 20260505171710",
    )
    .execute(pool)
    .await
    .expect("Failed to copy core migrations");

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
    .execute(pool)
    .await
    .expect("Failed to create desktop migrations table");

    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
         SELECT version, description, installed_on, success, checksum, execution_time
         FROM _sqlx_migrations_legacy_backup
         WHERE version < 20260505171710",
    )
    .execute(pool)
    .await
    .expect("Failed to copy desktop migrations");

    // Drop the backup
    sqlx::query("DROP TABLE _sqlx_migrations_legacy_backup")
        .execute(pool)
        .await
        .expect("Failed to drop legacy backup table");

    log::info!("Successfully split legacy migration table");
}

pub async fn run_migrations<R: tauri::Runtime>(app_handle: AppHandle<R>, pool: SqlitePool) {
    // ── Legacy migration split (one-time for existing databases) ──
    migrate_legacy_migrations_table(&pool).await;

    // Run aether-core (library) migrations with custom table `_sqlx_migrations_aether_core`
    migrations::run_migrations(&pool).await;

    // Run desktop (application) migrations in the default `_sqlx_migrations` table
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Desktop migration failed");

    state::migrate(app_handle, pool)
        .await
        .expect("State migration failed");
}
