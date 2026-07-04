use std::sync::Arc;

use async_trait::async_trait;
use sqlx::SqlitePool;
use sqlx::pool::PoolOptions;
use tempfile::TempDir;
use uuid::Uuid;

use crate::features::instance::infra::{
    FsInstanceFileService, SqliteInstanceStorage, SqlitePackStorage,
};
use crate::features::instance::{
    Instance, InstanceBuilder, InstanceError, InstanceStorage, InstanceWatcherService,
    RemoveInstanceUseCase,
};
use crate::features::minecraft::ModLoader;
use crate::features::settings::LocationInfo;

// ─── Minimal inline mock ───────────────────────────────────────

struct NoOpWatcher;

#[async_trait]
impl InstanceWatcherService for NoOpWatcher {
    async fn watch_instances(&self) -> Result<(), InstanceError> {
        Ok(())
    }
    async fn watch_instance(&self, _instance_id: &str) -> Result<(), InstanceError> {
        Ok(())
    }
    async fn unwatch_instance(&self, _instance_id: &str) -> Result<(), InstanceError> {
        Ok(())
    }
}

// ─── End-to-end test ───────────────────────────────────────────

async fn create_test_infrastructure() -> (TempDir, Arc<LocationInfo>, SqlitePool) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let location_info = Arc::new(LocationInfo::new(
        temp_dir.path().join("settings"),
        temp_dir.path().to_path_buf(),
    ));

    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = PoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to create in-memory SQLite pool");

    // Create tables (same as delete_instance_test)
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS instances (
          id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL, icon_path TEXT,
          install_stage TEXT NOT NULL, game_version TEXT NOT NULL, loader TEXT NOT NULL,
          loader_version_json TEXT,
          override_java_path BOOLEAN NOT NULL DEFAULT 0, java_path TEXT NOT NULL DEFAULT '',
          override_launch_args BOOLEAN NOT NULL DEFAULT 0, launch_args_json TEXT NOT NULL DEFAULT '[]',
          override_env_vars BOOLEAN NOT NULL DEFAULT 0, env_vars_json TEXT NOT NULL DEFAULT '[]',
          override_memory BOOLEAN NOT NULL DEFAULT 0, memory_maximum INTEGER NOT NULL DEFAULT 2048,
          override_window_settings BOOLEAN NOT NULL DEFAULT 0, force_fullscreen BOOLEAN NOT NULL DEFAULT 0,
          window_width INTEGER NOT NULL DEFAULT 960, window_height INTEGER NOT NULL DEFAULT 540,
          created_at DATETIME NOT NULL, modified_at DATETIME NOT NULL, last_played_at DATETIME,
          time_played INTEGER NOT NULL DEFAULT 0, recent_time_played INTEGER NOT NULL DEFAULT 0,
          override_hooks BOOLEAN NOT NULL DEFAULT 0, hook_pre_launch TEXT NOT NULL DEFAULT '',
          hook_wrapper TEXT NOT NULL DEFAULT '', hook_post_exit TEXT NOT NULL DEFAULT ''
        )
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create instances table");

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS instance_pack_info (
          instance_id TEXT PRIMARY KEY NOT NULL, provider_id TEXT NOT NULL,
          modpack_id TEXT NOT NULL, version_id TEXT NOT NULL,
          FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
        )
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create instance_pack_info table");

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS packs (
          instance_id TEXT PRIMARY KEY,
          FOREIGN KEY(instance_id) REFERENCES instances(id) ON DELETE CASCADE
        )
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create packs table");

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS pack_files (
          instance_id TEXT NOT NULL, content_path TEXT NOT NULL,
          file_name TEXT NOT NULL, name TEXT, hash TEXT NOT NULL, side TEXT, update_provider_id TEXT,
          PRIMARY KEY(instance_id, content_path),
          FOREIGN KEY(instance_id) REFERENCES packs(instance_id) ON DELETE CASCADE
        )
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create pack_files table");

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS pack_file_updates (
          instance_id TEXT NOT NULL, content_path TEXT NOT NULL,
          provider_id TEXT NOT NULL, content_id TEXT NOT NULL, version_id TEXT NOT NULL,
          PRIMARY KEY(instance_id, content_path, provider_id),
          FOREIGN KEY(instance_id, content_path) REFERENCES pack_files(instance_id, content_path) ON DELETE CASCADE
        )
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create pack_file_updates table");

    (temp_dir, location_info, pool)
}

fn create_instance(id: &str) -> Instance {
    InstanceBuilder::new(
        id.to_string(),
        "Integration Test".to_string(),
        "1.21".to_string(),
        ModLoader::Fabric,
    )
    .build()
}

async fn seed_pack_data(pool: &SqlitePool, instance_id: &str) {
    sqlx::query("INSERT INTO packs (instance_id) VALUES (?1)")
        .bind(instance_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES (?1, 'mods/a.jar', 'a.jar', 'h1')",
    )
    .bind(instance_id)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) VALUES (?1, 'mods/a.jar', 'modrinth', 'abc', '1.0')",
    )
    .bind(instance_id)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO instance_pack_info (instance_id, provider_id, modpack_id, version_id) VALUES (?1, 'curseforge', 'pack1', '1.0')",
    )
    .bind(instance_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn create_instance_dir(location_info: &LocationInfo, instance_id: &str) {
    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    tokio::fs::write(dir.join("mods").join("test.jar"), b"data")
        .await
        .unwrap();
    tokio::fs::write(dir.join("options.txt"), b"options")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_remove_use_case_cleans_everything() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    // ── Arrange: real services with real infrastructure ──
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let file_service = Arc::new(FsInstanceFileService::new(location_info.clone()));
    let watcher = Arc::new(NoOpWatcher);

    let instance = create_instance(&instance_id);
    storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;
    create_instance_dir(&location_info, &instance_id).await;

    let use_case = RemoveInstanceUseCase::new(storage, watcher, file_service, pack_storage);

    // ── Act ──
    use_case.execute(instance_id.clone()).await.unwrap();

    // ── Assert: DB completely clean ──
    for table in &[
        "instances",
        "instance_pack_info",
        "packs",
        "pack_files",
        "pack_file_updates",
    ] {
        let query = format!("SELECT COUNT(*) as cnt FROM {table}");
        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);
        assert_eq!(count, 0, "Table {table} should be empty after deletion");
    }

    // ── Assert: directory removed from disk ──
    let instance_dir = location_info.instance_dir(&instance_id);
    assert!(
        !instance_dir.exists(),
        "Instance directory should be removed"
    );
}

#[tokio::test]
async fn test_remove_use_case_nonexistent_instance() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let file_service = Arc::new(FsInstanceFileService::new(location_info.clone()));
    let watcher = Arc::new(NoOpWatcher);

    let use_case = RemoveInstanceUseCase::new(storage, watcher, file_service, pack_storage);

    // Deleting something that never existed should be fine
    use_case.execute(instance_id).await.unwrap();
}

#[tokio::test]
async fn test_remove_use_case_reuses_pool() {
    // Ensures all three SQL-based services work on the same pool/connection
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let file_service = Arc::new(FsInstanceFileService::new(location_info.clone()));
    let watcher = Arc::new(NoOpWatcher);

    let instance = create_instance(&instance_id);
    storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;

    let use_case = RemoveInstanceUseCase::new(storage, watcher, file_service, pack_storage);

    use_case.execute(instance_id.clone()).await.unwrap();

    // Verify: the same pool sees no data
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "All instances gone on the shared pool");
}
