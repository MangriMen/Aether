use sqlx::{SqlitePool, migrate::MigrateError};

/// Run all `SQLx` migrations for the `aether-core` library.
///
/// Uses the custom migrations table `_sqlx_migrations_aether_core` (configured
/// via `sqlx.toml`) so that library migrations are completely independent from
/// application-level migrations, even when both target the same database.
///
/// Fails with [`MigrateError::VersionMissing`] when the database was migrated by a newer
/// version of the library.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
