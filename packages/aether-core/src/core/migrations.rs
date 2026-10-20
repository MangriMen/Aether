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

#[cfg(test)]
mod tests {
    use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};

    const PACKS: &str = include_str!("../../migrations/20260507192000_packs.sql");
    const INSTALLED_BY: &str =
        include_str!("../../migrations/20261013120000_pack_files_installed_by.sql");
    const SLASHES: &str =
        include_str!("../../migrations/20261020120000_pack_files_forward_slashes.sql");

    /// The pack tables as they were before the migration, with foreign keys enforced as in the app.
    async fn pool_before() -> SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(true);
        let pool = sqlx::pool::PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        for sql in [
            "CREATE TABLE instances (id TEXT PRIMARY KEY)",
            PACKS,
            INSTALLED_BY,
            "INSERT INTO instances (id) VALUES ('i')",
            "INSERT INTO packs (instance_id) VALUES ('i')",
        ] {
            sqlx::raw_sql(sql).execute(&pool).await.unwrap();
        }
        pool
    }

    /// In a transaction, as the migrator runs it: the deferred foreign keys last until its commit.
    async fn apply_slashes(pool: &SqlitePool) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql(SLASHES).execute(&mut *tx).await.unwrap();
        tx.commit().await.unwrap();
    }

    async fn add_file(pool: &SqlitePool, path: &str, hash: &str) {
        sqlx::query(
            "INSERT INTO pack_files (instance_id, content_path, file_name, hash)
             VALUES ('i', ?, 'a.jar', ?)",
        )
        .bind(path)
        .bind(hash)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO pack_file_updates
             (instance_id, content_path, provider_id, content_id, version_id)
             VALUES ('i', ?, 'p#c', 'content', ?)",
        )
        .bind(path)
        .bind(hash)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn rows(pool: &SqlitePool, table: &str) -> Vec<(String, String)> {
        let query = if table == "pack_files" {
            "SELECT content_path, hash FROM pack_files ORDER BY content_path"
        } else {
            "SELECT content_path, version_id FROM pack_file_updates ORDER BY content_path"
        };
        sqlx::query(query)
            .fetch_all(pool)
            .await
            .unwrap()
            .iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect()
    }

    #[tokio::test]
    async fn should_rename_backslash_paths_with_their_updates() {
        let pool = pool_before().await;
        add_file(&pool, r"mods\a.jar", "h1").await;
        add_file(&pool, "mods/b.jar", "h2").await;

        apply_slashes(&pool).await;

        let expected = vec![
            ("mods/a.jar".to_owned(), "h1".to_owned()),
            ("mods/b.jar".to_owned(), "h2".to_owned()),
        ];
        assert_eq!(rows(&pool, "pack_files").await, expected);
        assert_eq!(rows(&pool, "pack_file_updates").await, expected);
    }

    #[tokio::test]
    async fn should_keep_the_slash_row_when_both_spellings_exist() {
        let pool = pool_before().await;
        add_file(&pool, r"mods\a.jar", "old").await;
        add_file(&pool, "mods/a.jar", "kept").await;

        apply_slashes(&pool).await;

        let expected = vec![("mods/a.jar".to_owned(), "kept".to_owned())];
        assert_eq!(rows(&pool, "pack_files").await, expected);
        assert_eq!(rows(&pool, "pack_file_updates").await, expected);
    }
}
