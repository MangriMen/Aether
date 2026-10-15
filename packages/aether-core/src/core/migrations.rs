use sqlx::SqlitePool;

/// Run all `SQLx` migrations for the `aether-core` library.
///
/// Uses the custom migrations table `_sqlx_migrations_aether_core` (configured
/// via `sqlx.toml`) so that library migrations are completely independent from
/// application-level migrations, even when both target the same database.
pub async fn run_migrations(pool: &SqlitePool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("aether-core migrations failed");
}
