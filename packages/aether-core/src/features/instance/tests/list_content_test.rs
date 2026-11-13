//! `ListContentUseCase` reading an instance's content folders.

use crate::features::instance::{
    ContentId, ContentPath, ContentVersionId, InstanceError, InstanceLocks, Pack, PackFile,
    PackStorage, PlanOrigin,
};
use crate::shared::{
    hash::{FileHash, HashAlgorithm},
    web_url::HttpsUrl,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::SystemTime,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::{
        events::{Event, EventEmitter, EventError},
        instance::{
            CachedContentAuthor, CachedContentIcon, CachedContentMetadata,
            CachedContentVersionMetadata, ContentAuthor, ContentIconCache, ContentIconError,
            ContentMetadata, ContentMetadataCache, ContentMetadataService, ContentProvider,
            ContentVersionMetadata, InstanceId, ListContentUseCase, ProviderId,
            infra::SqlitePackStorage,
        },
        settings::LocationInfo,
    },
    shared::capability::infra::MemoryCapabilityRegistry,
};

struct NoCache;

#[async_trait]
impl ContentMetadataCache for NoCache {
    async fn get_many(
        &self,
        _: &ProviderId,
        _: &[ContentId],
    ) -> HashMap<ContentId, CachedContentMetadata> {
        HashMap::new()
    }

    async fn put_keyed_at(
        &self,
        _: &ProviderId,
        _: &[(ContentId, ContentMetadata)],
        _: SystemTime,
    ) -> bool {
        true
    }

    async fn get_versions(
        &self,
        _: &ProviderId,
        _: &[ContentVersionId],
    ) -> HashMap<ContentVersionId, CachedContentVersionMetadata> {
        HashMap::new()
    }

    async fn put_versions(&self, _: &ProviderId, _: &[ContentVersionMetadata]) -> bool {
        true
    }

    async fn get_authors(
        &self,
        _: &ProviderId,
        _: &[String],
    ) -> HashMap<String, CachedContentAuthor> {
        HashMap::new()
    }

    async fn put_authors(&self, _: &ProviderId, _: &[ContentAuthor]) -> bool {
        true
    }
}

struct NoIcons;

#[async_trait]
impl ContentIconCache for NoIcons {
    async fn lookup(&self, _: &HttpsUrl) -> CachedContentIcon {
        CachedContentIcon::Missing
    }

    async fn fetch(&self, _: &HttpsUrl) -> Result<bool, ContentIconError> {
        Err(ContentIconError::NotAnImage)
    }

    async fn retain(&self, _: &std::collections::HashSet<HttpsUrl>) -> usize {
        0
    }
}

struct NoEvents;

#[async_trait]
impl EventEmitter<Event> for NoEvents {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

#[tokio::test]
async fn should_list_the_content_there_is_when_some_content_folders_are_missing() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let mods = location_info
        .instance_dir(&InstanceId::try_from("instance").unwrap())
        .join("mods");
    tokio::fs::create_dir_all(&mods).await.unwrap();
    tokio::fs::write(mods.join("a.jar"), b"jar").await.unwrap();
    let use_case = ListContentUseCase::new(
        Arc::new(SqlitePackStorage::in_memory().await),
        location_info,
        Arc::new(ContentMetadataService::new(
            Arc::new(NoCache),
            Arc::new(NoIcons),
            Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default()),
            Arc::new(NoEvents),
        )),
        Arc::new(InstanceLocks::new()),
    );

    let files = use_case
        .execute(InstanceId::try_from("instance").unwrap())
        .await
        .unwrap();

    assert_eq!(files.len(), 1);
    assert!(files.contains_key("mods/a.jar"));
}

#[tokio::test]
async fn should_list_the_enabled_file_when_it_sits_next_to_its_disabled_copy() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let mods = location_info
        .instance_dir(&InstanceId::try_from("instance").unwrap())
        .join("mods");
    tokio::fs::create_dir_all(&mods).await.unwrap();
    tokio::fs::write(mods.join("a.jar"), b"enabled")
        .await
        .unwrap();
    tokio::fs::write(mods.join("a.jar.disabled"), b"disabled")
        .await
        .unwrap();
    let use_case = ListContentUseCase::new(
        Arc::new(SqlitePackStorage::in_memory().await),
        location_info,
        Arc::new(ContentMetadataService::new(
            Arc::new(NoCache),
            Arc::new(NoIcons),
            Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default()),
            Arc::new(NoEvents),
        )),
        Arc::new(InstanceLocks::new()),
    );

    for _ in 0..5 {
        let files = use_case
            .execute(InstanceId::try_from("instance").unwrap())
            .await
            .unwrap();

        assert_eq!(files.len(), 1);
        let file = files.get("mods/a.jar").unwrap();
        assert!(!file.file.disabled);
    }
}

#[tokio::test]
async fn should_strip_only_one_disabled_suffix_when_listing() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let mods = location_info
        .instance_dir(&InstanceId::try_from("instance").unwrap())
        .join("mods");
    tokio::fs::create_dir_all(&mods).await.unwrap();
    tokio::fs::write(mods.join("a.jar.disabled.disabled"), b"disabled")
        .await
        .unwrap();
    let use_case = ListContentUseCase::new(
        Arc::new(SqlitePackStorage::in_memory().await),
        location_info,
        Arc::new(ContentMetadataService::new(
            Arc::new(NoCache),
            Arc::new(NoIcons),
            Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default()),
            Arc::new(NoEvents),
        )),
        Arc::new(InstanceLocks::new()),
    );

    let files = use_case
        .execute(InstanceId::try_from("instance").unwrap())
        .await
        .unwrap();

    assert_eq!(files.len(), 1);
    let file = files.get("mods/a.jar.disabled").unwrap();
    assert!(file.file.disabled);
}

/// Storage that can answer `get_pack` with no files, as a listing's snapshot does when a plan
/// records a file only afterwards, and counts the reads of a single file's record.
struct SpyStorage {
    inner: SqlitePackStorage,
    stale: bool,
    single_reads: AtomicUsize,
}

impl SpyStorage {
    fn new(inner: SqlitePackStorage, stale: bool) -> Self {
        Self {
            inner,
            stale,
            single_reads: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl PackStorage for SpyStorage {
    async fn get_pack(&self, id: &InstanceId) -> Result<Pack, InstanceError> {
        if self.stale {
            return Ok(Pack::default());
        }
        self.inner.get_pack(id).await
    }

    async fn find_pack_file(
        &self,
        id: &InstanceId,
        path: &ContentPath,
    ) -> Result<Option<PackFile>, InstanceError> {
        self.single_reads.fetch_add(1, Ordering::SeqCst);
        self.inner.find_pack_file(id, path).await
    }

    async fn get_pack_files(
        &self,
        id: &InstanceId,
    ) -> Result<HashMap<ContentPath, PackFile>, InstanceError> {
        if self.stale {
            return Ok(HashMap::new());
        }
        self.inner.get_pack_files(id).await
    }

    async fn update_pack(&self, id: &InstanceId, pack: &Pack) -> Result<(), InstanceError> {
        self.inner.update_pack(id, pack).await
    }

    async fn get_pack_file(
        &self,
        id: &InstanceId,
        path: &ContentPath,
    ) -> Result<PackFile, InstanceError> {
        self.single_reads.fetch_add(1, Ordering::SeqCst);
        self.inner.get_pack_file(id, path).await
    }

    async fn insert_pack_file_if_absent(
        &self,
        id: &InstanceId,
        path: &ContentPath,
        file: &PackFile,
    ) -> Result<PackFile, InstanceError> {
        self.inner.insert_pack_file_if_absent(id, path, file).await
    }

    async fn find_by_provider_id(
        &self,
        id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &ContentId,
    ) -> Result<Option<PackFile>, InstanceError> {
        self.inner
            .find_by_provider_id(id, provider_id, content_id)
            .await
    }

    async fn update_pack_file(
        &self,
        id: &InstanceId,
        path: &ContentPath,
        file: &PackFile,
    ) -> Result<(), InstanceError> {
        self.inner.update_pack_file(id, path, file).await
    }

    async fn update_pack_file_many(
        &self,
        id: &InstanceId,
        paths: &[ContentPath],
        files: &[PackFile],
    ) -> Result<(), InstanceError> {
        self.inner.update_pack_file_many(id, paths, files).await
    }

    async fn remove_pack_file(
        &self,
        id: &InstanceId,
        path: &ContentPath,
    ) -> Result<(), InstanceError> {
        self.inner.remove_pack_file(id, path).await
    }

    async fn remove_pack_file_many(
        &self,
        id: &InstanceId,
        paths: &[ContentPath],
    ) -> Result<(), InstanceError> {
        self.inner.remove_pack_file_many(id, paths).await
    }

    async fn remove_all_for_instance(&self, id: &InstanceId) -> Result<(), InstanceError> {
        self.inner.remove_all_for_instance(id).await
    }
}

#[tokio::test]
async fn should_keep_the_owner_of_a_file_a_plan_recorded_after_the_listing_took_its_snapshot() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let instance_id = InstanceId::try_from("instance").unwrap();
    let mods = location_info.instance_dir(&instance_id).join("mods");
    tokio::fs::create_dir_all(&mods).await.unwrap();
    tokio::fs::write(mods.join("a.jar"), b"jar").await.unwrap();
    let storage = SqlitePackStorage::in_memory().await;
    let path = ContentPath::parse("mods/a.jar").unwrap();
    let origin = PlanOrigin {
        provider_id: "plugin#cap".parse().unwrap(),
        content_id: ContentId::try_from("pack").unwrap(),
    };
    let planned = PackFile {
        name: Some("Sodium".into()),
        installed_by: Some(origin.clone()),
        ..PackFile::from_hash(
            "a.jar".into(),
            FileHash::new(HashAlgorithm::Sha1, "0".repeat(40)).unwrap(),
        )
    };
    storage
        .update_pack_file(&instance_id, &path, &planned)
        .await
        .unwrap();
    let storage = Arc::new(SpyStorage::new(storage, true));
    let use_case = ListContentUseCase::new(
        storage.clone(),
        location_info,
        Arc::new(ContentMetadataService::new(
            Arc::new(NoCache),
            Arc::new(NoIcons),
            Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default()),
            Arc::new(NoEvents),
        )),
        Arc::new(InstanceLocks::new()),
    );

    use_case.execute(instance_id.clone()).await.unwrap();

    let stored = storage.get_pack_file(&instance_id, &path).await.unwrap();
    assert_eq!(stored.name.as_deref(), Some("Sodium"));
    assert_eq!(stored.installed_by.unwrap().content_id, origin.content_id);
}

fn use_case_for(
    storage: Arc<dyn PackStorage>,
    location_info: Arc<LocationInfo>,
    locks: Arc<InstanceLocks>,
) -> ListContentUseCase {
    ListContentUseCase::new(
        storage,
        location_info,
        Arc::new(ContentMetadataService::new(
            Arc::new(NoCache),
            Arc::new(NoIcons),
            Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default()),
            Arc::new(NoEvents),
        )),
        locks,
    )
}

async fn instance_with_jars(names: &[&str]) -> (TempDir, Arc<LocationInfo>, InstanceId) {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let instance_id = InstanceId::try_from("instance").unwrap();
    let mods = location_info.instance_dir(&instance_id).join("mods");
    tokio::fs::create_dir_all(&mods).await.unwrap();
    for name in names {
        tokio::fs::write(mods.join(name), name.as_bytes())
            .await
            .unwrap();
    }
    (temp, location_info, instance_id)
}

#[tokio::test]
async fn should_not_read_the_records_one_by_one_when_listing_recorded_files() {
    let names = ["a.jar", "b.jar", "c.jar", "d.jar"];
    let (_temp, location_info, instance_id) = instance_with_jars(&names).await;
    let storage = Arc::new(SpyStorage::new(SqlitePackStorage::in_memory().await, false));
    let use_case = use_case_for(
        storage.clone(),
        location_info,
        Arc::new(InstanceLocks::new()),
    );
    use_case.execute(instance_id.clone()).await.unwrap();
    storage.single_reads.store(0, Ordering::SeqCst);

    let files = use_case.execute(instance_id).await.unwrap();

    assert_eq!(files.len(), names.len());
    assert_eq!(storage.single_reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn should_list_a_file_without_recording_it_when_the_instance_is_locked() {
    let (_temp, location_info, instance_id) = instance_with_jars(&["a.jar"]).await;
    let storage = Arc::new(SqlitePackStorage::in_memory().await);
    let locks = Arc::new(InstanceLocks::new());
    let use_case = use_case_for(storage.clone(), location_info, locks.clone());

    let held = locks.try_lock(&instance_id).unwrap();
    let files = use_case.execute(instance_id.clone()).await.unwrap();
    assert!(files.contains_key("mods/a.jar"));
    assert!(
        storage
            .get_pack(&instance_id)
            .await
            .unwrap()
            .files
            .is_empty()
    );

    drop(held);
    use_case.execute(instance_id.clone()).await.unwrap();
    assert_eq!(storage.get_pack(&instance_id).await.unwrap().files.len(), 1);
}

#[tokio::test]
async fn should_record_every_new_file_in_one_pass_when_the_instance_is_free() {
    let names: Vec<String> = (0..40).map(|n| format!("m{n}.jar")).collect();
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let (_temp, location_info, instance_id) = instance_with_jars(&name_refs).await;
    let storage = Arc::new(SqlitePackStorage::in_memory().await);
    let use_case = use_case_for(
        storage.clone(),
        location_info,
        Arc::new(InstanceLocks::new()),
    );

    use_case.execute(instance_id.clone()).await.unwrap();

    assert_eq!(
        storage.get_pack(&instance_id).await.unwrap().files.len(),
        names.len()
    );
}
