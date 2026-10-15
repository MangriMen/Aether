//! `ListContentUseCase` reading an instance's content folders.

use std::{collections::HashMap, sync::Arc, time::SystemTime};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::{
        events::{Event, EventEmitter, EventError},
        instance::{
            CachedContentIcon, CachedContentMetadata, CachedContentVersionMetadata,
            ContentIconCache, ContentIconError, ContentMetadata, ContentMetadataCache,
            ContentMetadataService, ContentProvider, ContentVersionMetadata, ListContentUseCase,
            ProviderId, infra::SqlitePackStorage,
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
        _: &[String],
    ) -> HashMap<String, CachedContentMetadata> {
        HashMap::new()
    }

    async fn put_keyed_at(
        &self,
        _: &ProviderId,
        _: &[(String, ContentMetadata)],
        _: SystemTime,
    ) -> bool {
        true
    }

    async fn get_versions(
        &self,
        _: &ProviderId,
        _: &[String],
    ) -> HashMap<String, CachedContentVersionMetadata> {
        HashMap::new()
    }

    async fn put_versions(&self, _: &ProviderId, _: &[ContentVersionMetadata]) -> bool {
        true
    }
}

struct NoIcons;

#[async_trait]
impl ContentIconCache for NoIcons {
    async fn lookup(&self, _: &str) -> CachedContentIcon {
        CachedContentIcon::Missing
    }

    async fn fetch(&self, _: &str) -> Result<bool, ContentIconError> {
        Err(ContentIconError::NotAnImage)
    }

    async fn remove_unused(&self, _: std::time::Duration) -> usize {
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
    let mods = location_info.instance_dir("instance").join("mods");
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

    let files = use_case.execute("instance".to_owned()).await.unwrap();

    assert_eq!(files.len(), 1);
    assert!(files.contains_key("mods/a.jar"));
}
