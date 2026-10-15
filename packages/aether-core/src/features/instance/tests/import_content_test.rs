//! `ImportContentUseCase` copying files into an instance under the instance's lock.

use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::{
    events::{Event, EventEmitter, EventError},
    instance::{
        ContentType, ImportContent, ImportContentUseCase, InstanceLocks, PackStorage,
        infra::SqlitePackStorage,
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
    let instance_mods = location_info.instance_dir(INSTANCE_ID).join("mods");
    tokio::fs::create_dir_all(&instance_mods).await.unwrap();
    let pack_storage = Arc::new(SqlitePackStorage::in_memory().await);
    let locks = Arc::new(InstanceLocks::new());
    let use_case = ImportContentUseCase::new(
        Arc::new(NoEvents),
        pack_storage.clone(),
        location_info,
        locks.clone(),
    );
    let held = locks.lock(INSTANCE_ID).await;

    let mut import = std::pin::pin!(use_case.execute(ImportContent::new(
        INSTANCE_ID.to_owned(),
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
            .get_pack_file(
                INSTANCE_ID,
                &std::path::Path::new("mods").join("a.jar").to_string_lossy()
            )
            .await
            .is_ok()
    );
}
