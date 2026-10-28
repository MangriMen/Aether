use crate::features::instance::infra::{SqliteInstanceStorage, SqlitePackStorage};
use crate::features::instance::{
    Instance, InstanceBuilder, InstanceId, InstanceStorage, PackStorage,
};
use crate::features::minecraft::ModLoader;
use sqlx::SqlitePool;

use sqlx::pool::PoolOptions;

// Integration tests for instance deletion with real `SQLite`.
//
// These tests verify that:
// - `ON DELETE CASCADE` works when `foreign_keys(true)` is enabled
// - `SqlitePackStorage::remove_all_for_instance` cleans up pack data
// - No orphaned rows are left behind after instance deletion
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

    crate::core::migrations::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Create a test instance with a given ID.
fn create_test_instance(id: &str) -> Instance {
    InstanceBuilder::new(
        InstanceId::try_from(id).unwrap(),
        "Test Instance".to_string(),
        "1.20.1".to_string(),
        ModLoader::Fabric,
    )
    .build()
}

/// Count rows in a table (runtime query, no compile-time check).
async fn count_rows(pool: &SqlitePool, table: &str) -> i64 {
    let query = format!("SELECT COUNT(*) as cnt FROM {table}");
    sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(query))
        .fetch_one(pool)
        .await
        .expect("Failed to count rows")
}

/// Count rows in a table filtered by `instance_id`.
async fn count_rows_for_instance(pool: &SqlitePool, table: &str, instance_id: &str) -> i64 {
    let query = format!("SELECT COUNT(*) as cnt FROM {table} WHERE instance_id = ?1");
    sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(query))
        .bind(instance_id)
        .fetch_one(pool)
        .await
        .expect("Failed to count rows for instance")
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[tokio::test]
async fn should_cascade_pack_info_when_an_instance_is_deleted_with_foreign_keys() {
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
    storage
        .remove(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();

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
async fn should_clean_up_explicitly_when_foreign_keys_are_off() {
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
        .remove_all_for_instance(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    instance_storage
        .remove(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();

    // Assert: cleanup works regardless of FK enforcement
    assert_eq!(count_rows(&pool, "instances").await, 0);
    assert_eq!(
        count_rows(&pool, "instance_pack_info").await,
        0,
        "Explicit cleanup works even without foreign_keys=ON"
    );
}

#[tokio::test]
async fn should_clean_the_pack_tables_when_all_data_of_an_instance_is_removed() {
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
        .remove_all_for_instance(&InstanceId::try_from(instance_id.as_str()).unwrap())
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
async fn should_leave_no_orphans_when_an_instance_is_deleted_completely() {
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
        .remove_all_for_instance(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    instance_storage
        .remove(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();

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
async fn should_clean_everything_up_when_foreign_keys_are_off() {
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
        .remove_all_for_instance(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    instance_storage
        .remove(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();

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
