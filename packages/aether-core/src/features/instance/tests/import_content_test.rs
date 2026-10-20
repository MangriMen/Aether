//! `ImportContentUseCase` copying files into an instance under the instance's lock.

use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::{
    events::{Event, EventEmitter, EventError},
    instance::{
        ContentType, ImportContent, ImportContentUseCase, InstanceError, InstanceId, InstanceLocks,
        Pack, PackFile, PackStorage, ProviderId, infra::SqlitePackStorage,
    },
    settings::LocationInfo,
};

const INSTANCE_ID: &str = "instance";

struct NoEvents;

#[async_trait]
impl EventEmitter<Event> for NoEvents {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

#[tokio::test]
async fn should_wait_for_the_instance_lock_when_a_plan_holds_it() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let source = temp.path().join("source").join("a.jar");
    tokio::fs::create_dir_all(source.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&source, b"jar").await.unwrap();
    let instance_mods = location_info
        .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
        .join("mods");
    tokio::fs::create_dir_all(&instance_mods).await.unwrap();
    let pack_storage = Arc::new(SqlitePackStorage::in_memory().await);
    let locks = Arc::new(InstanceLocks::new());
    let use_case = ImportContentUseCase::new(
        Arc::new(NoEvents),
        pack_storage.clone(),
        location_info,
        locks.clone(),
    );
    let held = locks
        .lock(&InstanceId::try_from(INSTANCE_ID).unwrap())
        .await;

    let mut import = std::pin::pin!(use_case.execute(ImportContent::new(
        InstanceId::try_from(INSTANCE_ID).unwrap(),
        ContentType::Mod,
        vec![source],
    )));
    assert!(futures::poll!(&mut import).is_pending());
    assert!(!instance_mods.join("a.jar").exists());

    drop(held);
    import.await.unwrap();
    assert!(instance_mods.join("a.jar").exists());
    assert!(
        pack_storage
            .get_pack_file(&InstanceId::try_from(INSTANCE_ID).unwrap(), "mods/a.jar")
            .await
            .is_ok()
    );
}

struct Setup {
    _temp: TempDir,
    mods: std::path::PathBuf,
    sources: std::path::PathBuf,
    pack_storage: Arc<SqlitePackStorage>,
    use_case: ImportContentUseCase,
}

fn instance_id() -> InstanceId {
    InstanceId::try_from(INSTANCE_ID).unwrap()
}

async fn setup_with(wrap: impl FnOnce(Arc<SqlitePackStorage>) -> Arc<dyn PackStorage>) -> Setup {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let mods = location_info.instance_dir(&instance_id()).join("mods");
    tokio::fs::create_dir_all(&mods).await.unwrap();
    let sources = temp.path().join("sources");
    tokio::fs::create_dir_all(sources.join("other"))
        .await
        .unwrap();
    let pack_storage = Arc::new(SqlitePackStorage::in_memory().await);
    let use_case = ImportContentUseCase::new(
        Arc::new(NoEvents),
        wrap(pack_storage.clone()),
        location_info,
        Arc::new(InstanceLocks::new()),
    );
    Setup {
        _temp: temp,
        mods,
        sources,
        pack_storage,
        use_case,
    }
}

async fn setup() -> Setup {
    setup_with(|storage| storage).await
}

impl Setup {
    async fn source(&self, name: &str, in_other: bool) -> std::path::PathBuf {
        let dir = if in_other {
            self.sources.join("other")
        } else {
            self.sources.clone()
        };
        let path = dir.join(name);
        tokio::fs::write(&path, name.as_bytes()).await.unwrap();
        path
    }

    async fn import(&self, sources: Vec<std::path::PathBuf>) -> Result<(), InstanceError> {
        self.use_case
            .execute(ImportContent::new(instance_id(), ContentType::Mod, sources))
            .await
    }
}

/// The pack records and the content listing key a file as `mods/a.jar` on every platform.
#[tokio::test]
async fn should_record_the_content_path_with_forward_slashes() {
    let setup = setup().await;
    let a = setup.source("a.jar", false).await;

    setup.import(vec![a]).await.unwrap();

    let files: Vec<_> = setup
        .pack_storage
        .get_pack(&instance_id())
        .await
        .unwrap()
        .files
        .into_iter()
        .map(|entry| entry.file)
        .collect();
    assert_eq!(files, ["mods/a.jar"]);
}

/// A file removed from the instance can be imported again: nothing of the first import is left
/// under a name the second one does not look at.
#[tokio::test]
async fn should_import_again_when_the_file_was_removed() {
    let setup = setup().await;
    let a = setup.source("a.jar", false).await;
    setup.import(vec![a.clone()]).await.unwrap();
    tokio::fs::remove_file(setup.mods.join("a.jar"))
        .await
        .unwrap();
    setup
        .pack_storage
        .remove_pack_file(&instance_id(), "mods/a.jar")
        .await
        .unwrap();

    setup.import(vec![a]).await.unwrap();

    assert!(setup.mods.join("a.jar").exists());
}

#[tokio::test]
async fn should_refuse_a_batch_when_two_files_share_a_name() {
    let setup = setup().await;
    let first = setup.source("a.jar", false).await;
    let second = setup.source("a.jar", true).await;

    let result = setup.import(vec![first, second]).await;

    assert!(matches!(
        result,
        Err(InstanceError::ContentDuplication { content_path }) if content_path == "mods/a.jar"
    ));
    assert!(!setup.mods.join("a.jar").exists());
}

#[tokio::test]
async fn should_refuse_the_import_when_the_file_exists_disabled() {
    let setup = setup().await;
    tokio::fs::write(setup.mods.join("a.jar.disabled"), b"old")
        .await
        .unwrap();
    let a = setup.source("a.jar", false).await;

    let result = setup.import(vec![a]).await;

    assert!(matches!(
        result,
        Err(InstanceError::ContentDuplication { .. })
    ));
    assert!(!setup.mods.join("a.jar").exists());
}

/// Delegates to the real storage but fails to record the files.
struct FailingRecord(Arc<SqlitePackStorage>);

#[async_trait]
impl PackStorage for FailingRecord {
    async fn get_pack(&self, id: &InstanceId) -> Result<Pack, InstanceError> {
        self.0.get_pack(id).await
    }
    async fn update_pack(&self, id: &InstanceId, pack: &Pack) -> Result<(), InstanceError> {
        self.0.update_pack(id, pack).await
    }
    async fn get_pack_file(&self, id: &InstanceId, path: &str) -> Result<PackFile, InstanceError> {
        self.0.get_pack_file(id, path).await
    }
    async fn find_by_provider_id(
        &self,
        id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &str,
    ) -> Result<Option<PackFile>, InstanceError> {
        self.0
            .find_by_provider_id(id, provider_id, content_id)
            .await
    }
    async fn update_pack_file(
        &self,
        id: &InstanceId,
        path: &str,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        self.0.update_pack_file(id, path, pack_file).await
    }
    async fn update_pack_file_many(
        &self,
        _: &InstanceId,
        _: &[String],
        _: &[PackFile],
    ) -> Result<(), InstanceError> {
        Err(InstanceError::Storage("disk full".into()))
    }
    async fn remove_pack_file(&self, id: &InstanceId, path: &str) -> Result<(), InstanceError> {
        self.0.remove_pack_file(id, path).await
    }
    async fn remove_pack_file_many(
        &self,
        id: &InstanceId,
        paths: &[String],
    ) -> Result<(), InstanceError> {
        self.0.remove_pack_file_many(id, paths).await
    }
    async fn remove_all_for_instance(&self, id: &InstanceId) -> Result<(), InstanceError> {
        self.0.remove_all_for_instance(id).await
    }
}

#[tokio::test]
async fn should_leave_no_files_when_recording_them_fails() {
    let setup = setup_with(|storage| Arc::new(FailingRecord(storage))).await;
    let a = setup.source("a.jar", false).await;
    let b = setup.source("b.jar", false).await;

    let result = setup.import(vec![a.clone(), b]).await;

    assert!(matches!(result, Err(InstanceError::Storage(_))));
    assert!(!setup.mods.join("a.jar").exists());
    assert!(!setup.mods.join("b.jar").exists());
}

/// The folder is gone, so the copy of the first file fails: nothing is left, and a later
/// import is not told the file is a duplicate.
#[tokio::test]
async fn should_leave_no_files_when_the_copy_fails() {
    let setup = setup().await;
    let a = setup.source("a.jar", false).await;
    tokio::fs::remove_dir_all(&setup.mods).await.unwrap();

    let result = setup.import(vec![a.clone()]).await;

    assert!(matches!(result, Err(InstanceError::Storage(_))));
    tokio::fs::create_dir_all(&setup.mods).await.unwrap();
    setup.import(vec![a]).await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn should_refuse_import_when_the_content_folder_is_a_link_out_of_the_instance() {
    let setup = setup().await;
    let outside = setup.sources.join("outside");
    tokio::fs::create_dir_all(&outside).await.unwrap();
    tokio::fs::remove_dir(&setup.mods).await.unwrap();
    std::os::unix::fs::symlink(&outside, &setup.mods).unwrap();
    let a = setup.source("a.jar", false).await;

    let result = setup.import(vec![a]).await;

    assert!(matches!(
        result,
        Err(InstanceError::PathEscapesInstance { .. })
    ));
    assert!(!outside.join("a.jar").exists());
}
