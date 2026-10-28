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

#[derive(Default)]
struct RecordingWatcher {
    calls: std::sync::Mutex<Vec<&'static str>>,
}

#[async_trait]
impl InstanceWatcherService for RecordingWatcher {
    async fn watch_instances(&self) -> Result<(), InstanceError> {
        Ok(())
    }
    async fn watch_instance(&self, _instance_id: &InstanceId) -> Result<(), InstanceError> {
        self.calls.lock().unwrap().push("watch");
        Ok(())
    }
    async fn unwatch_instance(&self, _instance_id: &InstanceId) -> Result<(), InstanceError> {
        self.calls.lock().unwrap().push("unwatch");
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
        storage.clone(),
        watcher,
        file_service,
        pack_storage,
        Arc::new(InstanceLocks::new()),
        no_op_reaper(storage),
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
            .expect("Failed to count rows");
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
        storage.clone(),
        watcher,
        file_service,
        pack_storage,
        Arc::new(InstanceLocks::new()),
        no_op_reaper(storage),
    );

    // Deleting something that never existed should be fine
    use_case
        .execute(InstanceId::try_from(instance_id.clone()).unwrap())
        .await
        .unwrap();

    let instances: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(instances, 0);
    assert!(
        !location_info
            .instance_dir(&InstanceId::try_from(instance_id).unwrap())
            .exists(),
        "removing a missing instance must not create its directory"
    );
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
        storage.clone(),
        Arc::new(NoOpWatcher),
        file_service,
        pack_storage,
        locks.clone(),
        no_op_reaper(storage),
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
        _base_id: &InstanceId,
    ) -> Result<(InstanceId, std::path::PathBuf), InstanceError> {
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
        storage.clone(),
        Arc::new(NoOpWatcher),
        Arc::new(FailingRemoveFileService),
        Arc::new(SqlitePackStorage::new(pool.clone())),
        Arc::new(InstanceLocks::new()),
        no_op_reaper(storage),
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

#[tokio::test]
async fn should_watch_the_instance_again_when_the_directory_cannot_be_removed() {
    let (_temp, _location_info, pool) = create_test_infrastructure().await;
    let instance_id = Uuid::new_v4().to_string();
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    storage
        .upsert(&create_instance(&instance_id))
        .await
        .unwrap();
    let watcher = Arc::new(RecordingWatcher::default());
    let use_case = RemoveInstanceUseCase::new(
        storage.clone(),
        watcher.clone(),
        Arc::new(FailingRemoveFileService),
        Arc::new(SqlitePackStorage::new(pool.clone())),
        Arc::new(InstanceLocks::new()),
        no_op_reaper(storage),
    );

    let result = use_case
        .execute(InstanceId::try_from(instance_id).unwrap())
        .await;

    assert!(result.is_err());
    assert_eq!(*watcher.calls.lock().unwrap(), vec!["unwatch", "watch"]);
}

// ─── Icon assets ───

#[derive(Default)]
struct RecordingAssets {
    removed: std::sync::Mutex<Vec<String>>,
}

#[async_trait]
impl crate::shared::cache::domain::AssetsStorage for RecordingAssets {
    async fn import_file(
        &self,
        source: &std::path::Path,
    ) -> Result<String, crate::shared::cache::domain::AssetError> {
        Ok(format!(
            "imported-{}",
            source.file_name().unwrap().to_string_lossy()
        ))
    }

    async fn remove(&self, id: &str) -> Result<(), crate::shared::cache::domain::AssetError> {
        self.removed.lock().unwrap().push(id.to_owned());
        Ok(())
    }
}

fn no_op_reaper(storage: Arc<SqliteInstanceStorage>) -> crate::features::instance::IconAssetReaper {
    crate::features::instance::IconAssetReaper::new(storage, Arc::new(RecordingAssets::default()))
}

async fn instance_with_icon(storage: &SqliteInstanceStorage, id: &str, icon: &str) {
    let mut instance = create_instance(id);
    instance.set_icon(Some(icon.to_owned()));
    storage.upsert(&instance).await.unwrap();
}

async fn remove_with_assets(
    storage: Arc<SqliteInstanceStorage>,
    pool: &SqlitePool,
    location_info: Arc<LocationInfo>,
    assets: Arc<RecordingAssets>,
    id: &str,
) {
    let use_case = RemoveInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoOpWatcher),
        Arc::new(FsInstanceFileService::new(
            location_info,
            crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
        )),
        Arc::new(SqlitePackStorage::new(pool.clone())),
        Arc::new(InstanceLocks::new()),
        crate::features::instance::IconAssetReaper::new(storage, assets),
    );
    use_case
        .execute(InstanceId::try_from(id).unwrap())
        .await
        .unwrap();
}

#[tokio::test]
async fn should_delete_the_icon_asset_when_its_last_instance_is_removed() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    instance_with_icon(&storage, "one", "icon.png").await;
    let assets = Arc::new(RecordingAssets::default());

    remove_with_assets(storage, &pool, location_info, assets.clone(), "one").await;

    assert_eq!(*assets.removed.lock().unwrap(), ["icon.png"]);
}

#[tokio::test]
async fn should_keep_the_icon_asset_when_another_instance_uses_it() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    instance_with_icon(&storage, "one", "icon.png").await;
    instance_with_icon(&storage, "two", "icon.png").await;
    let assets = Arc::new(RecordingAssets::default());

    remove_with_assets(storage, &pool, location_info, assets.clone(), "one").await;

    assert!(assets.removed.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_delete_the_old_icon_asset_when_the_icon_is_replaced() {
    use crate::features::instance::{EditInstanceIcon, EditInstanceIconUseCase};

    let (_temp, _location_info, pool) = create_test_infrastructure().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    instance_with_icon(&storage, "one", "old.png").await;
    let assets = Arc::new(RecordingAssets::default());
    let use_case = EditInstanceIconUseCase::new(
        storage.clone(),
        crate::features::instance::IconAssetReaper::new(storage.clone(), assets.clone()),
    );

    let updated = use_case
        .execute(EditInstanceIcon {
            instance_id: InstanceId::try_from("one").unwrap(),
            icon_path: Some(Some("new.png".to_owned())),
        })
        .await
        .unwrap();

    assert_eq!(updated.icon_path(), Some("imported-new.png"));
    assert_eq!(*assets.removed.lock().unwrap(), ["old.png"]);
}

#[tokio::test]
async fn should_keep_the_icon_asset_when_the_same_icon_is_set_again() {
    use crate::features::instance::{EditInstanceIcon, EditInstanceIconUseCase};

    let (_temp, _location_info, pool) = create_test_infrastructure().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    instance_with_icon(&storage, "one", "imported-same.png").await;
    let assets = Arc::new(RecordingAssets::default());
    let use_case = EditInstanceIconUseCase::new(
        storage.clone(),
        crate::features::instance::IconAssetReaper::new(storage.clone(), assets.clone()),
    );

    use_case
        .execute(EditInstanceIcon {
            instance_id: InstanceId::try_from("one").unwrap(),
            icon_path: Some(Some("same.png".to_owned())),
        })
        .await
        .unwrap();

    assert!(assets.removed.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_not_release_an_icon_while_it_is_being_imported() {
    let (_temp, _location_info, pool) = create_test_infrastructure().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    let assets = Arc::new(RecordingAssets::default());
    let reaper = crate::features::instance::IconAssetReaper::new(storage, assets.clone());

    let guard = reaper.hold().await;
    let release = tokio::spawn({
        let reaper = reaper.clone();
        async move { reaper.release("icon.png").await }
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        !release.is_finished(),
        "release must wait for the icon in use"
    );
    assert!(assets.removed.lock().unwrap().is_empty());

    drop(guard);
    release.await.unwrap();
    assert_eq!(*assets.removed.lock().unwrap(), ["icon.png"]);
}

struct FailingRemoveAssets;

#[async_trait]
impl crate::shared::cache::domain::AssetsStorage for FailingRemoveAssets {
    async fn import_file(
        &self,
        _source: &std::path::Path,
    ) -> Result<String, crate::shared::cache::domain::AssetError> {
        unimplemented!()
    }

    async fn remove(&self, _id: &str) -> Result<(), crate::shared::cache::domain::AssetError> {
        Err(crate::shared::cache::domain::AssetError::Remove(
            "locked".to_owned(),
        ))
    }
}

#[tokio::test]
async fn should_not_fail_the_removal_when_the_icon_asset_cannot_be_deleted() {
    let (_temp, location_info, pool) = create_test_infrastructure().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool.clone()));
    instance_with_icon(&storage, "one", "icon.png").await;
    let use_case = RemoveInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoOpWatcher),
        Arc::new(FsInstanceFileService::new(
            location_info,
            crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
        )),
        Arc::new(SqlitePackStorage::new(pool.clone())),
        Arc::new(InstanceLocks::new()),
        crate::features::instance::IconAssetReaper::new(storage, Arc::new(FailingRemoveAssets)),
    );

    use_case
        .execute(InstanceId::try_from("one").unwrap())
        .await
        .unwrap();
}
