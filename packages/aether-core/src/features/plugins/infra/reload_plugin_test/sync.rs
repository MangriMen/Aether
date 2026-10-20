//! Syncing the registry with the plugins on disk while some are enabled.

use std::{
    collections::HashMap,
    sync::{Arc, atomic::Ordering},
};

use async_trait::async_trait;

use crate::features::plugins::{
    ExtractedPlugin, Plugin, PluginError, PluginState, PluginStorage, PluginSyncService,
    SyncPluginsUseCase,
};

use super::common::*;

/// The plugins found on disk, as the sync lists them.
struct OnDisk(HashMap<String, Plugin>);

#[async_trait]
impl PluginStorage for OnDisk {
    async fn add(&self, _: ExtractedPlugin) -> Result<(), PluginError> {
        unimplemented!("the sync only lists")
    }
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError> {
        Ok(self.0.clone())
    }
    async fn get(&self, _: &str) -> Result<Plugin, PluginError> {
        unimplemented!("the sync only lists")
    }
    async fn remove(&self, _: &str) -> Result<(), PluginError> {
        unimplemented!("the sync only lists")
    }
    async fn replace(&self, _: ExtractedPlugin) -> Result<(), PluginError> {
        unimplemented!("the sync only lists")
    }
    async fn restore_replaced(&self, _: &str) -> Result<(), PluginError> {
        unimplemented!("the sync only lists")
    }
    async fn discard_replaced(&self, _: &str) -> Result<(), PluginError> {
        unimplemented!("the sync only lists")
    }
    async fn wasm_sha256(&self, _: &str) -> Result<String, PluginError> {
        unimplemented!("the sync only lists")
    }
}

impl Harness {
    /// A sync that finds `on_disk`.
    fn sync(&self, on_disk: HashMap<String, Plugin>) -> SyncPluginsUseCase {
        SyncPluginsUseCase::new(
            Arc::new(OnDisk(on_disk)),
            Arc::clone(&self.plugins),
            Arc::new(self.disable()),
            Arc::new(self.enable()),
            self.settings.clone(),
            Arc::new(NoOpEmitter),
        )
    }

    /// The plugin as it is registered, with another build behind it.
    fn rebuilt(&self) -> HashMap<String, Plugin> {
        let mut plugin = self.plugins.get(PLUGIN_ID).unwrap().clone();
        plugin.hash = "another build".into();
        plugin.state = PluginState::NotLoaded;
        HashMap::from([(PLUGIN_ID.to_owned(), plugin)])
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_run_the_new_build_and_keep_the_plugin_enabled_when_its_files_changed() {
    let harness = Harness::enabled().await;

    harness.sync(harness.rebuilt()).execute().await.unwrap();

    assert_eq!(
        harness.plugins.get(PLUGIN_ID).unwrap().hash,
        "another build"
    );
    assert!(matches!(harness.state(), PluginState::Loaded(_)));
    assert!(
        harness.is_enabled(),
        "an update must not disable the plugin"
    );
    assert_eq!(harness.loader.unloads.load(Ordering::SeqCst), 1);
    assert_eq!(
        harness.loader.loads.load(Ordering::SeqCst),
        2,
        "the first enable of the harness and the new build"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_leave_the_plugin_disabled_when_a_disabled_plugin_changed() {
    let harness = Harness::enabled().await;
    harness.disable().execute(PLUGIN_ID.into()).await.unwrap();

    harness.sync(harness.rebuilt()).execute().await.unwrap();

    assert_eq!(
        harness.plugins.get(PLUGIN_ID).unwrap().hash,
        "another build"
    );
    assert!(matches!(harness.state(), PluginState::NotLoaded));
    assert!(!harness.is_enabled());
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_forget_the_plugin_as_enabled_when_its_files_are_gone() {
    let harness = Harness::enabled().await;

    harness.sync(HashMap::new()).execute().await.unwrap();

    assert!(harness.plugins.get(PLUGIN_ID).is_err());
    assert!(!harness.is_enabled());
}
