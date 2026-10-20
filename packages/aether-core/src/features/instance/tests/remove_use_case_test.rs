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
    Instance, InstanceBuilder, InstanceError, InstanceId, InstanceLocks, InstanceStorage,
    InstanceWatcherService, RemoveInstanceUseCase, RemoveInstanceUseCasePort,
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
    async fn watch_instance(&self, _instance_id: &InstanceId) -> Result<(), InstanceError> {
        Ok(())
    }
    async fn unwatch_instance(&self, _instance_id: &InstanceId) -> Result<(), InstanceError> {
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
        InstanceId::try_from(id).unwrap(),
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
    let dir = location_info.instance_dir(&InstanceId::try_from(instance_id).unwrap());
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
async fn should_clean_everything_when_an_instance_is_removed() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    // ── Arrange: real services with real infrastructure ──
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let file_service = Arc::new(FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    ));
    let watcher = Arc::new(NoOpWatcher);

    let instance = create_instance(&instance_id);
    storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;
    create_instance_dir(&location_info, &instance_id).await;

    let use_case = RemoveInstanceUseCase::new(
        storage,
        watcher,
        file_service,
        pack_storage,
        Arc::new(InstanceLocks::new()),
    );

    // ── Act ──
    use_case
        .execute(InstanceId::try_from(instance_id.clone()).unwrap())
        .await
        .unwrap();

    // ── Assert: DB completely clean ──
    for table in &[
        "instances",
        "instance_pack_info",
        "packs",
        "pack_files",
        "pack_file_updates",
    ] {
        let query = format!("SELECT COUNT(*) as cnt FROM {table}");
        let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
            .fetch_one(&pool)
            .await
            .unwrap_or(0);
        assert_eq!(count, 0, "Table {table} should be empty after deletion");
    }

    // ── Assert: directory removed from disk ──
    let instance_dir =
        location_info.instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap());
    assert!(
        !instance_dir.exists(),
        "Instance directory should be removed"
    );
}

#[tokio::test]
async fn should_succeed_when_the_instance_to_remove_does_not_exist() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let file_service = Arc::new(FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    ));
    let watcher = Arc::new(NoOpWatcher);

    let use_case = RemoveInstanceUseCase::new(
        storage,
        watcher,
        file_service,
        pack_storage,
        Arc::new(InstanceLocks::new()),
    );

    // Deleting something that never existed should be fine
    use_case
        .execute(InstanceId::try_from(instance_id).unwrap())
        .await
        .unwrap();
}

#[tokio::test]
async fn should_reuse_the_pool_when_an_instance_is_removed() {
    // Ensures all three SQL-based services work on the same pool/connection
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let file_service = Arc::new(FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    ));
    let watcher = Arc::new(NoOpWatcher);

    let instance = create_instance(&instance_id);
    storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;

    let use_case = RemoveInstanceUseCase::new(
        storage,
        watcher,
        file_service,
        pack_storage,
        Arc::new(InstanceLocks::new()),
    );

    use_case
        .execute(InstanceId::try_from(instance_id.clone()).unwrap())
        .await
        .unwrap();

    // Verify: the same pool sees no data
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "All instances gone on the shared pool");
}

#[tokio::test]
async fn should_wait_for_the_instance_lock_when_a_plan_holds_it_on_removal() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let file_service = Arc::new(FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    ));
    storage
        .upsert(&create_instance(&instance_id))
        .await
        .unwrap();
    create_instance_dir(&location_info, &instance_id).await;
    let locks = Arc::new(InstanceLocks::new());
    let use_case = RemoveInstanceUseCase::new(
        storage,
        Arc::new(NoOpWatcher),
        file_service,
        pack_storage,
        locks.clone(),
    );
    let held = locks
        .lock(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await;

    let mut remove =
        std::pin::pin!(use_case.execute(InstanceId::try_from(instance_id.clone()).unwrap()));
    assert!(futures::poll!(&mut remove).is_pending());
    assert!(
        location_info
            .instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap())
            .exists()
    );

    drop(held);
    remove.await.unwrap();
    assert!(
        !location_info
            .instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap())
            .exists()
    );
}

struct FailingRemoveFileService;

#[async_trait]
impl crate::features::instance::InstanceFileService for FailingRemoveFileService {
    async fn create_instance_dir(&self, _id: &InstanceId) -> Result<(), InstanceError> {
        Ok(())
    }
    async fn create_unique_instance_dir(
        &self,
        _base_name: &str,
    ) -> Result<(String, std::path::PathBuf), InstanceError> {
        unimplemented!()
    }
    async fn remove_instance_dir(&self, _id: &InstanceId) -> Result<(), InstanceError> {
        Err(InstanceError::Storage("locked".to_string()))
    }
    async fn write_instance_file(
        &self,
        _id: &InstanceId,
        _relative_path: &str,
        _bytes: &[u8],
    ) -> Result<(), InstanceError> {
        unimplemented!()
    }
    async fn read_instance_file(
        &self,
        _id: &InstanceId,
        _relative_path: &str,
    ) -> Result<Vec<u8>, InstanceError> {
        unimplemented!()
    }
}

#[tokio::test]
async fn should_keep_records_when_the_instance_directory_cannot_be_removed() {
    let (_temp, _location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    storage
        .upsert(&create_instance(&instance_id))
        .await
        .unwrap();
    seed_pack_data(&pool, &instance_id).await;
    let use_case = RemoveInstanceUseCase::new(
        storage,
        Arc::new(NoOpWatcher),
        Arc::new(FailingRemoveFileService),
        Arc::new(SqlitePackStorage::new(pool.clone())),
        Arc::new(InstanceLocks::new()),
    );

    let result = use_case
        .execute(InstanceId::try_from(instance_id).unwrap())
        .await;

    assert!(result.is_err());
    for table in ["instances", "packs", "pack_files"] {
        let count: i64 =
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1, "{table} must survive a failed directory removal");
    }
}
