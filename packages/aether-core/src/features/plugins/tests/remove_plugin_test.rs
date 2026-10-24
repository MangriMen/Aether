use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::features::plugins::{
    ExtractedPlugin, Plugin, PluginDisableService, PluginError, PluginStorage, PluginSyncService,
    RemovePluginUseCase,
};

type Log = Arc<Mutex<Vec<&'static str>>>;

struct FakeStorage(Log);

#[async_trait]
impl PluginStorage for FakeStorage {
    async fn add(&self, _: ExtractedPlugin) -> Result<(), PluginError> {
        unimplemented!()
    }
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError> {
        unimplemented!()
    }
    async fn get(&self, _: &str) -> Result<Plugin, PluginError> {
        unimplemented!()
    }
    async fn remove(&self, _: &str) -> Result<(), PluginError> {
        self.0.lock().unwrap().push("remove");
        Ok(())
    }
    async fn replace(&self, _: ExtractedPlugin) -> Result<(), PluginError> {
        unimplemented!()
    }
    async fn restore_replaced(&self, _: &str) -> Result<(), PluginError> {
        unimplemented!()
    }
    async fn discard_replaced(&self, _: &str) -> Result<(), PluginError> {
        unimplemented!()
    }
    async fn wasm_sha256(&self, _: &str) -> Result<String, PluginError> {
        unimplemented!()
    }
}

struct FakeDisable {
    log: Log,
    result: fn() -> Result<(), PluginError>,
}

#[async_trait]
impl PluginDisableService for FakeDisable {
    async fn execute(&self, _: String) -> Result<(), PluginError> {
        self.log.lock().unwrap().push("disable");
        (self.result)()
    }
}

struct FakeSync(Log);

#[async_trait]
impl PluginSyncService for FakeSync {
    async fn execute(&self) -> Result<(), PluginError> {
        self.0.lock().unwrap().push("sync");
        Ok(())
    }
}

fn use_case(result: fn() -> Result<(), PluginError>) -> (RemovePluginUseCase, Log) {
    let log = Log::default();
    let use_case = RemovePluginUseCase::new(
        Arc::new(FakeStorage(log.clone())),
        Arc::new(FakeDisable {
            log: log.clone(),
            result,
        }),
        Arc::new(FakeSync(log.clone())),
    );
    (use_case, log)
}

#[tokio::test]
async fn should_unload_the_plugin_before_deleting_its_files() {
    let (use_case, log) = use_case(|| Ok(()));

    use_case.execute("p".into()).await.unwrap();

    assert_eq!(*log.lock().unwrap(), ["disable", "remove", "sync"]);
}

#[tokio::test]
async fn should_keep_the_files_when_the_plugin_is_busy() {
    let (use_case, log) = use_case(|| {
        Err(PluginError::Busy {
            plugin_id: "p".into(),
        })
    });

    let result = use_case.execute("p".into()).await;

    assert!(matches!(result, Err(PluginError::Busy { .. })));
    assert_eq!(*log.lock().unwrap(), ["disable"]);
}

#[tokio::test]
async fn should_remove_a_plugin_that_was_not_loaded() {
    let (use_case, log) = use_case(|| {
        Err(PluginError::AlreadyUnloaded {
            plugin_id: "p".into(),
        })
    });

    use_case.execute("p".into()).await.unwrap();

    assert_eq!(*log.lock().unwrap(), ["disable", "remove", "sync"]);
}
