use crate::features::instance::infra::{SqliteInstanceStorage, SqlitePackStorage};
use crate::features::instance::{Instance, InstanceBuilder, InstanceStorage, PackStorage};
use crate::features::minecraft::ModLoader;
use sqlx::SqlitePool;

use sqlx::pool::PoolOptions;

/// Integration tests for instance deletion with real `SQLite`.
///
/// These tests verify that:
/// - `ON DELETE CASCADE` works when `foreign_keys(true)` is enabled
/// - `SqlitePackStorage::remove_all_for_instance` cleans up pack data
/// - No orphaned rows are left behind after instance deletion
///
/// Note: helpers here don't need `#[cfg(test)]` — the entire module
/// is gated by `#[cfg(test)]` in `parent/mod.rs`.
///
/// Create all required tables for instance deletion tests.
async fn run_migrations(pool: &SqlitePool) {
    // instances + instance_pack_info (from 20260505171728_instances.sql)
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS instances (
          id TEXT PRIMARY KEY NOT NULL,
          name TEXT NOT NULL,
          icon_path TEXT,
          install_stage TEXT NOT NULL,
          game_version TEXT NOT NULL,
          loader TEXT NOT NULL,
          loader_version_json TEXT,
          override_java_path BOOLEAN NOT NULL DEFAULT 0,
          java_path TEXT NOT NULL DEFAULT '',
          override_launch_args BOOLEAN NOT NULL DEFAULT 0,
          launch_args_json TEXT NOT NULL DEFAULT '[]',
          override_env_vars BOOLEAN NOT NULL DEFAULT 0,
          env_vars_json TEXT NOT NULL DEFAULT '[]',
          override_memory BOOLEAN NOT NULL DEFAULT 0,
          memory_maximum INTEGER NOT NULL DEFAULT 2048,
          override_window_settings BOOLEAN NOT NULL DEFAULT 0,
          force_fullscreen BOOLEAN NOT NULL DEFAULT 0,
          window_width INTEGER NOT NULL DEFAULT 960,
          window_height INTEGER NOT NULL DEFAULT 540,
          created_at DATETIME NOT NULL,
          modified_at DATETIME NOT NULL,
          last_played_at DATETIME,
          time_played INTEGER NOT NULL DEFAULT 0,
          recent_time_played INTEGER NOT NULL DEFAULT 0,
          override_hooks BOOLEAN NOT NULL DEFAULT 0,
          hook_pre_launch TEXT NOT NULL DEFAULT '',
          hook_wrapper TEXT NOT NULL DEFAULT '',
          hook_post_exit TEXT NOT NULL DEFAULT ''
        )
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create instances table");

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS instance_pack_info (
          instance_id TEXT PRIMARY KEY NOT NULL,
          provider_id TEXT NOT NULL,
          modpack_id TEXT NOT NULL,
          version_id TEXT NOT NULL,
          FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
        )
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create instance_pack_info table");

    // packs + pack_files + pack_file_updates (from 20260507192000_packs.sql)
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS packs (
          instance_id TEXT PRIMARY KEY,
          FOREIGN KEY(instance_id) REFERENCES instances(id) ON DELETE CASCADE
        )
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create packs table");

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS pack_files (
          instance_id TEXT NOT NULL,
          content_path TEXT NOT NULL,
          file_name TEXT NOT NULL,
          name TEXT,
          hash TEXT NOT NULL,
          side TEXT,
          update_provider_id TEXT,
          PRIMARY KEY(instance_id, content_path),
          FOREIGN KEY(instance_id) REFERENCES packs(instance_id) ON DELETE CASCADE
        )
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create pack_files table");

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS pack_file_updates (
          instance_id TEXT NOT NULL,
          content_path TEXT NOT NULL,
          provider_id TEXT NOT NULL,
          content_id TEXT NOT NULL,
          version_id TEXT NOT NULL,
          PRIMARY KEY(instance_id, content_path, provider_id),
          FOREIGN KEY(instance_id, content_path) REFERENCES pack_files(instance_id, content_path) ON DELETE CASCADE
        )
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create pack_file_updates table");
}

/// Create an in-memory `SQLite` pool with migrations applied.
/// Uses `max_connections(1)` to ensure all queries share the same in-memory database.
async fn create_pool(with_foreign_keys: bool) -> SqlitePool {
    let mut options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);

    if with_foreign_keys {
        options = options.foreign_keys(true);
    }

    let pool = PoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to create in-memory SQLite pool");

    run_migrations(&pool).await;

    pool
}

/// Create a test instance with a given ID.
fn create_test_instance(id: &str) -> Instance {
    InstanceBuilder::new(
        id.to_string(),
        "Test Instance".to_string(),
        "1.20.1".to_string(),
        ModLoader::Fabric,
    )
    .build()
}

/// Count rows in a table (runtime query, no compile-time check).
async fn count_rows(pool: &SqlitePool, table: &str) -> i64 {
    let query = format!("SELECT COUNT(*) as cnt FROM {table}");
    sqlx::query_scalar::<_, i64>(&query)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

/// Count rows in a table filtered by `instance_id`.
async fn count_rows_for_instance(pool: &SqlitePool, table: &str, instance_id: &str) -> i64 {
    let query = format!("SELECT COUNT(*) as cnt FROM {table} WHERE instance_id = ?1");
    sqlx::query_scalar::<_, i64>(&query)
        .bind(instance_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[tokio::test]
async fn test_instance_delete_with_foreign_keys_cascades_pack_info() {
    let pool = create_pool(true).await;
    let storage = SqliteInstanceStorage::new(pool.clone());
    let instance_id = uuid::Uuid::new_v4().to_string();

    // Create and save instance
    let instance = create_test_instance(&instance_id);
    storage.upsert(&instance).await.unwrap();

    // Verify: instance exists
    assert_eq!(count_rows(&pool, "instances").await, 1);
    assert_eq!(
        count_rows_for_instance(&pool, "instance_pack_info", &instance_id).await,
        0,
        "No pack_info initially"
    );

    // Insert pack_info directly
    sqlx::query("INSERT INTO instance_pack_info (instance_id, provider_id, modpack_id, version_id) VALUES (?1, 'test', 'pack1', 'v1')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        count_rows_for_instance(&pool, "instance_pack_info", &instance_id).await,
        1
    );

    // Act: delete the instance
    storage.remove(&instance_id).await.unwrap();

    // Assert: instance is gone
    assert_eq!(count_rows(&pool, "instances").await, 0);

    // Assert: pack_info was cascade-deleted
    assert_eq!(
        count_rows_for_instance(&pool, "instance_pack_info", &instance_id).await,
        0,
        "instance_pack_info should be cascade-deleted when foreign_keys=ON"
    );
}

#[tokio::test]
async fn test_instance_delete_without_foreign_keys_still_works_with_explicit_cleanup() {
    let pool = create_pool(false).await;
    let instance_storage = SqliteInstanceStorage::new(pool.clone());
    let pack_storage = SqlitePackStorage::new(pool.clone());
    let instance_id = uuid::Uuid::new_v4().to_string();

    let instance = create_test_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();

    // Insert pack_info
    sqlx::query("INSERT INTO instance_pack_info (instance_id, provider_id, modpack_id, version_id) VALUES (?1, 'test', 'pack1', 'v1')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    // Act: use the full cleanup flow — pack_storage + instance_storage
    pack_storage
        .remove_all_for_instance(&instance_id)
        .await
        .unwrap();
    instance_storage.remove(&instance_id).await.unwrap();

    // Assert: cleanup works regardless of FK enforcement
    assert_eq!(count_rows(&pool, "instances").await, 0);
    assert_eq!(
        count_rows(&pool, "instance_pack_info").await,
        0,
        "Explicit cleanup works even without foreign_keys=ON"
    );
}

#[tokio::test]
async fn test_remove_all_for_instance_cleans_pack_tables() {
    let pool = create_pool(true).await;
    let instance_storage = SqliteInstanceStorage::new(pool.clone());
    let pack_storage = SqlitePackStorage::new(pool.clone());
    let instance_id = uuid::Uuid::new_v4().to_string();

    // Create instance + packs data
    let instance = create_test_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();

    // Insert packs data directly
    sqlx::query("INSERT INTO packs (instance_id) VALUES (?1)")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES (?1, 'mods/test.jar', 'test.jar', 'abc123')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES (?1, 'mods/test.jar', 'modrinth', 'abcdef', '1.0.0')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(
        count_rows_for_instance(&pool, "packs", &instance_id).await,
        1
    );
    assert_eq!(
        count_rows_for_instance(&pool, "pack_files", &instance_id).await,
        1
    );
    assert_eq!(
        count_rows_for_instance(&pool, "pack_file_updates", &instance_id).await,
        1
    );

    // Act: remove all pack data
    pack_storage
        .remove_all_for_instance(&instance_id)
        .await
        .unwrap();

    // Assert: all pack-related rows are cleaned up
    assert_eq!(
        count_rows_for_instance(&pool, "packs", &instance_id).await,
        0,
        "packs should be cleaned up by remove_all_for_instance"
    );
    assert_eq!(
        count_rows_for_instance(&pool, "pack_files", &instance_id).await,
        0,
        "pack_files should be cleaned up by remove_all_for_instance"
    );
    assert_eq!(
        count_rows_for_instance(&pool, "pack_file_updates", &instance_id).await,
        0,
        "pack_file_updates should be cleaned up by remove_all_for_instance"
    );

    // Instance itself should still exist
    assert_eq!(count_rows(&pool, "instances").await, 1);
}

#[tokio::test]
async fn test_full_instance_deletion_no_orphans() {
    let pool = create_pool(true).await;
    let instance_storage = SqliteInstanceStorage::new(pool.clone());
    let pack_storage = SqlitePackStorage::new(pool.clone());
    let instance_id = uuid::Uuid::new_v4().to_string();

    // Create instance with all related data
    let instance = create_test_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();

    // pack_info
    sqlx::query("INSERT INTO instance_pack_info (instance_id, provider_id, modpack_id, version_id) VALUES (?1, 'curseforge', 'pack1', '1.0')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    // packs hierarchy
    sqlx::query("INSERT INTO packs (instance_id) VALUES (?1)")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES (?1, 'mods/a.jar', 'a.jar', 'hash1')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES (?1, 'mods/a.jar', 'modrinth', 'content1', '1.0')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    // Verify initial state
    assert_eq!(count_rows(&pool, "instances").await, 1);
    assert_eq!(
        count_rows_for_instance(&pool, "instance_pack_info", &instance_id).await,
        1
    );
    assert_eq!(
        count_rows_for_instance(&pool, "packs", &instance_id).await,
        1
    );
    assert_eq!(
        count_rows_for_instance(&pool, "pack_files", &instance_id).await,
        1
    );
    assert_eq!(
        count_rows_for_instance(&pool, "pack_file_updates", &instance_id).await,
        1
    );

    // Act: full deletion flow (order matches RemoveInstanceUseCase)
    pack_storage
        .remove_all_for_instance(&instance_id)
        .await
        .unwrap();
    instance_storage.remove(&instance_id).await.unwrap();

    // Assert: all tables are clean
    assert_eq!(count_rows(&pool, "instances").await, 0);
    assert_eq!(
        count_rows(&pool, "instance_pack_info").await,
        0,
        "No orphaned rows in instance_pack_info"
    );
    assert_eq!(
        count_rows(&pool, "packs").await,
        0,
        "No orphaned rows in packs"
    );
    assert_eq!(
        count_rows(&pool, "pack_files").await,
        0,
        "No orphaned rows in pack_files"
    );
    assert_eq!(
        count_rows(&pool, "pack_file_updates").await,
        0,
        "No orphaned rows in pack_file_updates"
    );
}

#[tokio::test]
async fn test_foreign_keys_off_full_cleanup_still_works() {
    let pool = create_pool(false).await;
    let instance_storage = SqliteInstanceStorage::new(pool.clone());
    let pack_storage = SqlitePackStorage::new(pool.clone());
    let instance_id = uuid::Uuid::new_v4().to_string();

    let instance = create_test_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();

    // All related data
    sqlx::query("INSERT INTO packs (instance_id) VALUES (?1)")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES (?1, 'mods/a.jar', 'a.jar', 'hash1')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES (?1, 'mods/a.jar', 'modrinth', 'content1', '1.0')")
        .bind(&instance_id)
        .execute(&pool)
        .await
        .unwrap();

    // Act: remove_all_for_instance (explicit cleanup) + storage.remove
    pack_storage
        .remove_all_for_instance(&instance_id)
        .await
        .unwrap();
    instance_storage.remove(&instance_id).await.unwrap();

    // Assert: still clean even without FK enforcement
    assert_eq!(count_rows(&pool, "instances").await, 0);
    assert_eq!(
        count_rows(&pool, "packs").await,
        0,
        "Explicit cleanup works even without foreign_keys"
    );
    assert_eq!(
        count_rows(&pool, "pack_files").await,
        0,
        "Explicit cleanup works even without foreign_keys"
    );
    assert_eq!(
        count_rows(&pool, "pack_file_updates").await,
        0,
        "Explicit cleanup works even without foreign_keys"
    );
}
