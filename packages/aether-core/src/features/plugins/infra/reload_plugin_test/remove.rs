//! Removing a plugin while it is enabled, restarted or synced.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    DisablePluginUseCase, EnablePluginUseCase, ExtractedPlugin, Plugin, PluginDisableService,
    PluginError, PluginState, PluginStorage, PluginSyncService, RemovePluginUseCase,
    SyncPluginsUseCase,
};

use super::common::*;

/// Lists what it holds and deletes nothing; says whether it was asked to, and can be made to
/// fail.
#[derive(Default)]
struct FakeStorage {
    on_disk: HashMap<PluginId, Plugin>,
    removes: AtomicUsize,
    fail_remove: AtomicBool,
}

#[async_trait]
impl PluginStorage for FakeStorage {
    async fn add(&self, _: ExtractedPlugin) -> Result<(), PluginError> {
        unimplemented!("the remove and the sync do not add")
    }
    async fn list(&self) -> Result<HashMap<PluginId, Plugin>, PluginError> {
        Ok(self.on_disk.clone())
    }
    async fn get(&self, _: &PluginId) -> Result<Plugin, PluginError> {
        unimplemented!("the remove and the sync do not get")
    }
    async fn remove(&self, plugin_id: &PluginId) -> Result<(), PluginError> {
        self.removes.fetch_add(1, Ordering::SeqCst);
        if self.fail_remove.load(Ordering::SeqCst) {
            return Err(PluginError::NotFound {
                plugin_id: plugin_id.to_string(),
            });
        }
        Ok(())
    }
    async fn replace(&self, _: ExtractedPlugin) -> Result<(), PluginError> {
        unimplemented!("the remove and the sync do not replace")
    }
    async fn restore_replaced(&self, _: &PluginId) -> Result<(), PluginError> {
        unimplemented!("the remove and the sync do not restore")
    }
    async fn discard_replaced(&self, _: &PluginId) -> Result<(), PluginError> {
        unimplemented!("the remove and the sync do not discard")
    }
    async fn wasm_sha256(&self, _: &PluginId) -> Result<String, PluginError> {
        unimplemented!("the remove and the sync do not hash")
    }
}

struct NoSync;

#[async_trait]
impl PluginSyncService for NoSync {
    async fn execute(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// A disable after which the user enables the plugin again, before the remove goes on.
struct DisableThenEnable {
    disable: DisablePluginUseCase,
    enable: EnablePluginUseCase,
}

#[async_trait]
impl PluginDisableService for DisableThenEnable {
    async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError> {
        self.disable.execute(plugin_id.clone()).await?;
        self.enable.execute(plugin_id).await
    }
    async fn unload(&self, _: PluginId) -> Result<(), PluginError> {
        unimplemented!("the remove ends the plugin for good")
    }
}

impl Harness {
    fn remove(&self, storage: Arc<FakeStorage>) -> RemovePluginUseCase {
        RemovePluginUseCase::new(
            Arc::clone(&self.plugins),
            storage,
            Arc::new(self.disable()),
            Arc::new(NoSync),
        )
    }

    fn sync_of(&self, storage: FakeStorage) -> SyncPluginsUseCase {
        SyncPluginsUseCase::new(
            Arc::new(storage),
            Arc::clone(&self.plugins),
            Arc::new(self.disable()),
            Arc::new(self.enable()),
            self.settings.clone(),
            Arc::new(NoOpEmitter),
        )
    }

    /// Disabled, and taken by a remove that has not finished.
    async fn being_removed() -> Self {
        let harness = Self::enabled().await;
        harness.disable().execute(plugin_id()).await.unwrap();
        harness.plugins.begin_removing(&plugin_id()).await.unwrap();
        harness
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_forget_the_plugin_when_it_is_removed() {
    let harness = Harness::enabled().await;
    let storage = Arc::new(FakeStorage::default());

    harness
        .remove(storage.clone())
        .execute(plugin_id())
        .await
        .unwrap();

    assert_eq!(storage.removes.load(Ordering::SeqCst), 1);
    assert!(harness.plugins.get(&plugin_id()).is_err());
    assert!(!harness.is_enabled());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_refuse_an_enable_when_the_plugin_is_being_removed() {
    let harness = Harness::being_removed().await;

    let refused = harness.enable().execute(plugin_id()).await;
    let forced = harness.force_enable().execute(plugin_id()).await;

    assert!(
        matches!(refused, Err(PluginError::UnloadingInProgress { .. })),
        "{refused:?}"
    );
    assert!(
        matches!(forced, Err(PluginError::UnloadingInProgress { .. })),
        "{forced:?}"
    );
    assert!(matches!(harness.state(), PluginState::Removing));
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_refuse_a_disable_when_the_plugin_is_being_removed() {
    let harness = Harness::being_removed().await;

    let refused = harness.disable().execute(plugin_id()).await;

    assert!(
        matches!(refused, Err(PluginError::UnloadingInProgress { .. })),
        "{refused:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_keep_the_files_when_an_enable_gets_in_before_the_remove() {
    let harness = Harness::enabled().await;
    let storage = Arc::new(FakeStorage::default());
    let remove = RemovePluginUseCase::new(
        Arc::clone(&harness.plugins),
        storage.clone(),
        Arc::new(DisableThenEnable {
            disable: harness.disable(),
            enable: harness.enable(),
        }),
        Arc::new(NoSync),
    );

    let refused = remove.execute(plugin_id()).await;

    assert!(
        matches!(refused, Err(PluginError::AlreadyLoaded { .. })),
        "{refused:?}"
    );
    assert_eq!(storage.removes.load(Ordering::SeqCst), 0);
    assert!(matches!(harness.state(), PluginState::Loaded(_)));
    assert!(harness.is_enabled());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_keep_the_files_when_the_plugin_is_loading_as_the_remove_begins() {
    let harness = Harness::enabled().await;
    harness.disable().execute(plugin_id()).await.unwrap();
    harness.loader.hold_loads.store(true, Ordering::SeqCst);
    let storage = Arc::new(FakeStorage::default());
    let harness = Arc::new(harness);

    let enable = tokio::spawn({
        let harness = Arc::clone(&harness);
        async move { harness.enable().execute(plugin_id()).await }
    });
    harness.loader.load_started.notified().await;

    let refused = harness.remove(storage.clone()).execute(plugin_id()).await;

    assert!(
        matches!(refused, Err(PluginError::LoadingInProgress { .. })),
        "{refused:?}"
    );
    assert_eq!(storage.removes.load(Ordering::SeqCst), 0);
    harness.loader.load_released.notify_one();
    enable.await.unwrap().unwrap();
    assert!(matches!(harness.state(), PluginState::Loaded(_)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_put_the_state_back_when_deleting_the_files_fails() {
    let harness = Harness::enabled().await;
    let storage = Arc::new(FakeStorage::default());
    storage.fail_remove.store(true, Ordering::SeqCst);

    let result = harness.remove(storage.clone()).execute(plugin_id()).await;

    assert!(result.is_err());
    assert!(
        matches!(harness.state(), PluginState::NotLoaded),
        "not left `Removing`, which would refuse every later enable"
    );
    harness.enable().execute(plugin_id()).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_leave_a_plugin_being_removed_to_the_remove_when_its_files_are_gone() {
    let harness = Harness::being_removed().await;

    harness
        .sync_of(FakeStorage::default())
        .execute()
        .await
        .unwrap();

    assert!(matches!(harness.state(), PluginState::Removing));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_put_a_plugin_being_removed_back_when_a_sync_still_finds_its_files() {
    let harness = Harness::being_removed().await;
    let mut plugin = harness.plugins.get(&plugin_id()).unwrap().clone();
    plugin.state = PluginState::NotLoaded;
    plugin.hash = "another build".into();
    let storage = FakeStorage {
        on_disk: HashMap::from([(plugin_id(), plugin)]),
        ..FakeStorage::default()
    };

    harness.sync_of(storage).execute().await.unwrap();

    assert!(matches!(harness.state(), PluginState::Removing));
    assert_eq!(harness.plugins.get(&plugin_id()).unwrap().hash, "hash");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_wait_for_a_settings_change_in_flight_when_the_plugin_is_removed() {
    let harness = Arc::new(Harness::enabled().await);
    let storage = Arc::new(FakeStorage::default());
    let settings_lock = harness.plugins.lock_settings(&plugin_id()).await;

    let remove = tokio::spawn({
        let harness = Arc::clone(&harness);
        let storage = storage.clone();
        async move { harness.remove(storage).execute(plugin_id()).await }
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert_eq!(
        storage.removes.load(Ordering::SeqCst),
        0,
        "the files are not deleted under a writer"
    );
    drop(settings_lock);
    remove.await.unwrap().unwrap();
    assert_eq!(storage.removes.load(Ordering::SeqCst), 1);
}
