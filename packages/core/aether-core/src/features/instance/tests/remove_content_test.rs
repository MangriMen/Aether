use std::sync::Arc;

use async_trait::async_trait;
use sqlx::SqlitePool;
use sqlx::pool::PoolOptions;
use tempfile::TempDir;
use uuid::Uuid;

use crate::features::events::{Event, EventEmitter, EventError, SharedEventEmitter};
use crate::features::instance::app::ContentFileService;
use crate::features::instance::infra::{
    FsContentFileService, SqliteInstanceStorage, SqlitePackStorage,
};
use crate::features::instance::{
    Instance, InstanceBuilder, InstanceStorage, PackStorage, RemoveContent, RemoveContentUseCase,
};
use crate::features::minecraft::ModLoader;
use crate::features::settings::LocationInfo;

// ─── Mock event emitter ────────────────────────────────────────

struct MockEventEmitter;

#[async_trait]
impl EventEmitter<Event> for MockEventEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {
        // no-op
    }
}

// ─── Test infrastructure ───────────────────────────────────────

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
          instance_id TEXT PRIMARY KEY NOT NULL,
          provider_id TEXT NOT NULL,
          modpack_id TEXT NOT NULL,
          version_id TEXT NOT NULL,
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
        "INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES (?1, 'mods/b.jar', 'b.jar', 'h2')",
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
}

async fn create_content_files(location_info: &LocationInfo, instance_id: &str) {
    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    tokio::fs::write(dir.join("mods/a.jar"), b"data")
        .await
        .unwrap();
    tokio::fs::write(dir.join("mods/b.jar"), b"more data")
        .await
        .unwrap();
}

// ─── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_remove_content_cleans_db_and_disk() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let event_emitter: SharedEventEmitter = Arc::new(MockEventEmitter);
    let instance_storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let content_file_service: Arc<dyn ContentFileService> =
        Arc::new(FsContentFileService::new(location_info.clone()));

    // Setup: instance + packs + content files on disk
    let instance = create_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;
    create_content_files(&location_info, &instance_id).await;

    // Verify initial state
    let pack = pack_storage.get_pack(&instance_id).await.unwrap();
    assert_eq!(pack.files.len(), 2, "Two pack files initially");

    let disk_dir = location_info.instance_dir(&instance_id);
    assert!(disk_dir.join("mods/a.jar").exists());
    assert!(disk_dir.join("mods/b.jar").exists());

    // Act — remove just mods/a.jar
    let use_case = RemoveContentUseCase::new(
        event_emitter,
        pack_storage.clone(),
        content_file_service.clone(),
    );

    use_case
        .execute(RemoveContent::multiple(
            instance_id.clone(),
            vec!["mods/a.jar".to_string()],
        ))
        .await
        .expect("RemoveContentUseCase should succeed");

    // Assert — DB record removed
    let pack = pack_storage.get_pack(&instance_id).await.unwrap();
    assert_eq!(pack.files.len(), 1, "One pack file should remain");
    assert_eq!(pack.files[0].file, "mods/b.jar", "Only b.jar should remain");

    // Assert — disk file removed, other files intact
    assert!(
        !disk_dir.join("mods/a.jar").exists(),
        "a.jar should be removed from disk"
    );
    assert!(
        disk_dir.join("mods/b.jar").exists(),
        "b.jar should still exist on disk"
    );
}

#[tokio::test]
async fn test_remove_content_multiple_files() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let event_emitter: SharedEventEmitter = Arc::new(MockEventEmitter);
    let instance_storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let content_file_service: Arc<dyn ContentFileService> =
        Arc::new(FsContentFileService::new(location_info.clone()));

    let instance = create_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;
    create_content_files(&location_info, &instance_id).await;

    // Act — remove both files
    let use_case = RemoveContentUseCase::new(
        event_emitter,
        pack_storage.clone(),
        content_file_service.clone(),
    );

    use_case
        .execute(RemoveContent::multiple(
            instance_id.clone(),
            vec!["mods/a.jar".to_string(), "mods/b.jar".to_string()],
        ))
        .await
        .expect("RemoveContentUseCase should succeed");

    // Assert — DB empty
    let pack = pack_storage.get_pack(&instance_id).await.unwrap();
    assert!(pack.files.is_empty(), "No pack files should remain");

    // Assert — disk clean
    let disk_dir = location_info.instance_dir(&instance_id);
    assert!(!disk_dir.join("mods/a.jar").exists());
    assert!(!disk_dir.join("mods/b.jar").exists());
}

#[tokio::test]
async fn test_remove_content_nonexistent_file_is_idempotent() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let event_emitter: SharedEventEmitter = Arc::new(MockEventEmitter);
    let instance_storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let content_file_service: Arc<dyn ContentFileService> =
        Arc::new(FsContentFileService::new(location_info.clone()));

    let instance = create_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;

    let use_case = RemoveContentUseCase::new(
        event_emitter,
        pack_storage.clone(),
        content_file_service.clone(),
    );

    // Removing a path not in the pack should not fail
    use_case
        .execute(RemoveContent::multiple(
            instance_id.clone(),
            vec!["mods/nonexistent.jar".to_string()],
        ))
        .await
        .expect("Should not fail for non-existent content");
}

#[tokio::test]
async fn test_remove_content_removes_only_selected_files() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let event_emitter: SharedEventEmitter = Arc::new(MockEventEmitter);
    let instance_storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let content_file_service: Arc<dyn ContentFileService> =
        Arc::new(FsContentFileService::new(location_info.clone()));

    let instance = create_instance(&instance_id);
    instance_storage.upsert(&instance).await.unwrap();
    seed_pack_data(&pool, &instance_id).await;
    create_content_files(&location_info, &instance_id).await;

    // Add a third file to pack and disk
    sqlx::query(
        "INSERT INTO pack_files (instance_id, content_path, file_name, hash) VALUES (?1, 'config/example.toml', 'example.toml', 'h3')",
    )
    .bind(&instance_id)
    .execute(&pool)
    .await
    .unwrap();

    let dir = location_info.instance_dir(&instance_id);
    tokio::fs::create_dir_all(dir.join("config"))
        .await
        .expect("Failed to create config dir");
    tokio::fs::write(dir.join("config/example.toml"), b"config data")
        .await
        .unwrap();

    // Act — remove only mods/a.jar
    let use_case = RemoveContentUseCase::new(
        event_emitter,
        pack_storage.clone(),
        content_file_service.clone(),
    );

    use_case
        .execute(RemoveContent::multiple(
            instance_id.clone(),
            vec!["mods/a.jar".to_string()],
        ))
        .await
        .expect("RemoveContentUseCase should succeed");

    // Assert — only a.jar removed
    assert!(!dir.join("mods/a.jar").exists(), "a.jar should be removed");
    assert!(dir.join("mods/b.jar").exists(), "b.jar should remain");
    assert!(
        dir.join("config/example.toml").exists(),
        "example.toml should remain"
    );

    let pack = pack_storage.get_pack(&instance_id).await.unwrap();
    let remaining: Vec<_> = pack.files.iter().map(|e| e.file.as_str()).collect();
    assert!(remaining.contains(&"mods/b.jar"), "b.jar still in pack DB");
    assert!(
        remaining.contains(&"config/example.toml"),
        "example.toml still in pack DB"
    );
    assert!(
        !remaining.contains(&"mods/a.jar"),
        "a.jar removed from pack DB"
    );
}
