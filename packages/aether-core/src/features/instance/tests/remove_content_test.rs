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
    Instance, InstanceBuilder, InstanceId, InstanceLocks, InstanceStorage, PackStorage,
    RemoveContent, RemoveContentUseCase,
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

    crate::core::migrations::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

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
    let dir = location_info.instance_dir(&InstanceId::try_from(instance_id).unwrap());
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
async fn should_clean_db_and_disk_when_content_is_removed() {
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
    let pack = pack_storage
        .get_pack(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    assert_eq!(pack.files.len(), 2, "Two pack files initially");

    let disk_dir = location_info.instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap());
    assert!(disk_dir.join("mods/a.jar").exists());
    assert!(disk_dir.join("mods/b.jar").exists());

    // Act — remove just mods/a.jar
    let use_case = RemoveContentUseCase::new(
        event_emitter,
        pack_storage.clone(),
        content_file_service.clone(),
        Arc::new(InstanceLocks::new()),
    );

    use_case
        .execute(RemoveContent::multiple(
            InstanceId::try_from(instance_id.clone()).unwrap(),
            vec!["mods/a.jar".to_string()],
        ))
        .await
        .expect("RemoveContentUseCase should succeed");

    // Assert — DB record removed
    let pack = pack_storage
        .get_pack(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
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
async fn should_remove_every_file_when_several_are_given() {
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
        Arc::new(InstanceLocks::new()),
    );

    use_case
        .execute(RemoveContent::multiple(
            InstanceId::try_from(instance_id.clone()).unwrap(),
            vec!["mods/a.jar".to_string(), "mods/b.jar".to_string()],
        ))
        .await
        .expect("RemoveContentUseCase should succeed");

    // Assert — DB empty
    let pack = pack_storage
        .get_pack(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    assert!(pack.files.is_empty(), "No pack files should remain");

    // Assert — disk clean
    let disk_dir = location_info.instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap());
    assert!(!disk_dir.join("mods/a.jar").exists());
    assert!(!disk_dir.join("mods/b.jar").exists());
}

#[tokio::test]
async fn should_succeed_when_the_file_to_remove_does_not_exist() {
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
        Arc::new(InstanceLocks::new()),
    );

    // Removing a path not in the pack should not fail
    use_case
        .execute(RemoveContent::multiple(
            InstanceId::try_from(instance_id.clone()).unwrap(),
            vec!["mods/nonexistent.jar".to_string()],
        ))
        .await
        .expect("Should not fail for non-existent content");
}

/// A file that cannot be removed (a directory at its path, on any OS) keeps its record; the
/// other paths are still removed.
#[tokio::test]
async fn should_keep_the_record_of_a_file_when_it_cannot_be_removed() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();

    let instance_storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    instance_storage
        .upsert(&create_instance(&instance_id))
        .await
        .unwrap();
    seed_pack_data(&pool, &instance_id).await;
    let dir = location_info.instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap());
    std::fs::create_dir_all(dir.join("mods/a.jar")).unwrap();
    std::fs::write(dir.join("mods/b.jar"), b"more data").unwrap();

    let use_case = RemoveContentUseCase::new(
        Arc::new(MockEventEmitter),
        pack_storage.clone(),
        Arc::new(FsContentFileService::new(location_info.clone())),
        Arc::new(InstanceLocks::new()),
    );
    let result = use_case
        .execute(RemoveContent::multiple(
            InstanceId::try_from(instance_id.clone()).unwrap(),
            vec!["mods/a.jar".to_string(), "mods/b.jar".to_string()],
        ))
        .await;

    assert!(result.is_err());
    assert!(dir.join("mods/a.jar").exists());
    assert!(!dir.join("mods/b.jar").exists());
    let pack = pack_storage
        .get_pack(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    let remaining: Vec<_> = pack.files.iter().map(|e| e.file.as_str()).collect();
    assert_eq!(remaining, ["mods/a.jar"]);
}

#[tokio::test]
async fn should_remove_only_the_selected_files_when_others_exist() {
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

    let dir = location_info.instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap());
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
        Arc::new(InstanceLocks::new()),
    );

    use_case
        .execute(RemoveContent::multiple(
            InstanceId::try_from(instance_id.clone()).unwrap(),
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

    let pack = pack_storage
        .get_pack(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
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

#[tokio::test]
async fn should_wait_for_the_instance_lock_when_a_plan_holds_it() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();
    let event_emitter: SharedEventEmitter = Arc::new(MockEventEmitter);
    let pack_storage = Arc::new(SqlitePackStorage::new(pool.clone()));
    let content_file_service: Arc<dyn ContentFileService> =
        Arc::new(FsContentFileService::new(location_info.clone()));
    SqliteInstanceStorage::new(pool.clone())
        .upsert(&create_instance(&instance_id))
        .await
        .unwrap();
    seed_pack_data(&pool, &instance_id).await;
    create_content_files(&location_info, &instance_id).await;
    let locks = Arc::new(InstanceLocks::new());
    let use_case = RemoveContentUseCase::new(
        event_emitter,
        pack_storage.clone(),
        content_file_service,
        locks.clone(),
    );
    let held = locks
        .lock(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await;

    let mut remove = std::pin::pin!(use_case.execute(RemoveContent::multiple(
        InstanceId::try_from(instance_id.clone()).unwrap(),
        vec!["mods/a.jar".to_string()],
    )));
    assert!(futures::poll!(&mut remove).is_pending());
    assert!(
        location_info
            .instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap())
            .join("mods/a.jar")
            .exists()
    );

    drop(held);
    remove.await.unwrap();
    assert_eq!(
        pack_storage
            .get_pack(&InstanceId::try_from(instance_id.as_str()).unwrap())
            .await
            .unwrap()
            .files
            .len(),
        1
    );
}
