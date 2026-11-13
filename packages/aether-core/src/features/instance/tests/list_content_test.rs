//! `ListContentUseCase` reading an instance's content folders.

use crate::features::instance::{
    ContentId, ContentPath, ContentVersionId, InstanceError, Pack, PackFile, PackStorage,
    PlanOrigin,
};
use crate::shared::{
    hash::{FileHash, HashAlgorithm},
    web_url::HttpsUrl,
};
use std::{collections::HashMap, sync::Arc, time::SystemTime};

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
    );

    let files = use_case
        .execute(InstanceId::try_from("instance").unwrap())
        .await
        .unwrap();

    assert_eq!(files.len(), 1);
    let file = files.get("mods/a.jar.disabled").unwrap();
    assert!(file.file.disabled);
}

/// Storage whose `get_pack` answers with no files, as a listing's snapshot does when a plan
/// records a file only afterwards.
struct StaleSnapshot(SqlitePackStorage);

#[async_trait]
impl PackStorage for StaleSnapshot {
    async fn get_pack(&self, _: &InstanceId) -> Result<Pack, InstanceError> {
        Ok(Pack::default())
    }

    async fn update_pack(&self, id: &InstanceId, pack: &Pack) -> Result<(), InstanceError> {
        self.0.update_pack(id, pack).await
    }

    async fn get_pack_file(
        &self,
        id: &InstanceId,
        path: &ContentPath,
    ) -> Result<PackFile, InstanceError> {
        self.0.get_pack_file(id, path).await
    }

    async fn insert_pack_file_if_absent(
        &self,
        id: &InstanceId,
        path: &ContentPath,
        file: &PackFile,
    ) -> Result<PackFile, InstanceError> {
        self.0.insert_pack_file_if_absent(id, path, file).await
    }

    async fn find_by_provider_id(
        &self,
        id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &ContentId,
    ) -> Result<Option<PackFile>, InstanceError> {
        self.0
            .find_by_provider_id(id, provider_id, content_id)
            .await
    }

    async fn update_pack_file(
        &self,
        id: &InstanceId,
        path: &ContentPath,
        file: &PackFile,
    ) -> Result<(), InstanceError> {
        self.0.update_pack_file(id, path, file).await
    }

    async fn update_pack_file_many(
        &self,
        id: &InstanceId,
        paths: &[ContentPath],
        files: &[PackFile],
    ) -> Result<(), InstanceError> {
        self.0.update_pack_file_many(id, paths, files).await
    }

    async fn remove_pack_file(
        &self,
        id: &InstanceId,
        path: &ContentPath,
    ) -> Result<(), InstanceError> {
        self.0.remove_pack_file(id, path).await
    }

    async fn remove_pack_file_many(
        &self,
        id: &InstanceId,
        paths: &[ContentPath],
    ) -> Result<(), InstanceError> {
        self.0.remove_pack_file_many(id, paths).await
    }

    async fn remove_all_for_instance(&self, id: &InstanceId) -> Result<(), InstanceError> {
        self.0.remove_all_for_instance(id).await
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
    let storage = Arc::new(StaleSnapshot(storage));
    let use_case = ListContentUseCase::new(
        storage.clone(),
        location_info,
        Arc::new(ContentMetadataService::new(
            Arc::new(NoCache),
            Arc::new(NoIcons),
            Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default()),
            Arc::new(NoEvents),
        )),
    );

    use_case.execute(instance_id.clone()).await.unwrap();

    let stored = storage.get_pack_file(&instance_id, &path).await.unwrap();
    assert_eq!(stored.name.as_deref(), Some("Sodium"));
    assert_eq!(stored.installed_by.unwrap().content_id, origin.content_id);
}
