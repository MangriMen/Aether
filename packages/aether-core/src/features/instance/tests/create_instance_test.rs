use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::{
    events::{Event, EventEmitter, EventError, SharedEventEmitter},
    instance::{
        CreateInstanceUseCase, Instance, InstanceError, InstanceFileService, InstanceId,
        InstanceInstallService, InstanceStorage, InstanceWatcherService, NewInstance,
    },
    minecraft::{LoaderVersionPreference, LoaderVersionService, ModLoader},
    settings::LocationInfo,
};
use crate::shared::cache::domain::{AssetError, AssetsStorage};

// ─── Inline mocks ──────────────────────────────────────────────

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

struct NoOpEmitter;

#[async_trait]
impl EventEmitter<Event> for NoOpEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }
    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

struct NoOpInstallService;

#[async_trait]
impl InstanceInstallService for NoOpInstallService {
    async fn execute(&self, _instance_id: InstanceId, _force: bool) -> Result<(), InstanceError> {
        Ok(())
    }
}

struct MockLoaderService {
    try_get_default_result: Option<LoaderVersionPreference>,
}

#[async_trait]
impl LoaderVersionService for MockLoaderService {
    async fn resolve(
        &self,
        _game_version: &str,
        _loader: &ModLoader,
        _loader_version: Option<&LoaderVersionPreference>,
    ) -> Result<
        Option<crate::features::minecraft::modded::LoaderVersion>,
        crate::features::minecraft::MinecraftDomainError,
    > {
        Ok(None)
    }

    async fn try_get_default(
        &self,
        _game_version: &str,
        _loader: &ModLoader,
    ) -> Result<Option<LoaderVersionPreference>, crate::features::minecraft::MinecraftDomainError>
    {
        Ok(self.try_get_default_result.clone())
    }
}

/// An in-memory `InstanceFileService` that creates dirs inside a `TempDir`.
struct MemInstanceFileService {
    instances_root: std::path::PathBuf,
}

impl MemInstanceFileService {
    fn new(instances_root: std::path::PathBuf) -> Self {
        Self { instances_root }
    }
}

#[async_trait]
impl InstanceFileService for MemInstanceFileService {
    async fn create_instance_dir(&self, _instance_id: &InstanceId) -> Result<(), InstanceError> {
        Ok(())
    }

    async fn create_unique_instance_dir(
        &self,
        base_name: &str,
    ) -> Result<(String, std::path::PathBuf), InstanceError> {
        let dir = self.instances_root.join(base_name);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        Ok((base_name.to_owned(), dir))
    }

    async fn remove_instance_dir(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let dir = self.instances_root.join(instance_id.as_str());
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir)
                .await
                .map_err(|e| InstanceError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    // Not exercised by these tests; the real path checks live in
    // `instance_relative_path_test` and `fs_instance_file_service_test`.
    async fn write_instance_file(
        &self,
        _instance_id: &InstanceId,
        _relative_path: &str,
        _bytes: &[u8],
    ) -> Result<(), InstanceError> {
        unimplemented!("create_instance does not write instance files")
    }

    async fn read_instance_file(
        &self,
        _instance_id: &InstanceId,
        _relative_path: &str,
    ) -> Result<Vec<u8>, InstanceError> {
        unimplemented!("create_instance does not read instance files")
    }
}

/// An in-memory `InstanceStorage` that keeps instances in a Vec.
struct MemInstanceStorage {
    instances: std::sync::Mutex<Vec<Instance>>,
}

impl MemInstanceStorage {
    fn new() -> Self {
        Self {
            instances: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl InstanceStorage for MemInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        let guard = self.instances.lock().unwrap();
        Ok(guard.clone())
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        let guard = self.instances.lock().unwrap();
        guard
            .iter()
            .find(|i| i.id() == id)
            .cloned()
            .ok_or_else(|| InstanceError::NotFound {
                instance_id: id.to_string(),
            })
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let mut guard = self.instances.lock().unwrap();
        if let Some(pos) = guard.iter().position(|i| i.id() == instance.id()) {
            guard[pos] = instance.clone();
        } else {
            guard.push(instance.clone());
        }
        Ok(())
    }

    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let mut guard = self.instances.lock().unwrap();
        guard.retain(|i| i.id() != id);
        Ok(())
    }
}

// ─── Test helpers ──────────────────────────────────────────────────────────────

fn setup_temp_location() -> (TempDir, Arc<LocationInfo>) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let location_info = Arc::new(LocationInfo::new(
        temp_dir.path().join("settings"),
        temp_dir.path().to_path_buf(),
    ));
    (temp_dir, location_info)
}

fn make_use_case(
    storage: Arc<dyn InstanceStorage>,
    loader_service: Arc<dyn LoaderVersionService>,
    install_service: Arc<dyn InstanceInstallService>,
    file_service: Arc<dyn InstanceFileService>,
    watcher: Arc<dyn InstanceWatcherService>,
    emitter: SharedEventEmitter,
) -> CreateInstanceUseCase {
    CreateInstanceUseCase::new(
        storage,
        loader_service,
        install_service,
        file_service,
        emitter,
        watcher,
        Arc::new(FakeAssets),
    )
}

/// Imports an existing file under the id `asset-<file name>`, like the real store names its copy.
struct FakeAssets;

#[async_trait]
impl AssetsStorage for FakeAssets {
    async fn import_file(&self, source: &Path) -> Result<String, AssetError> {
        if !source.is_file() {
            return Err(AssetError::ReadSource("no such file".to_owned()));
        }
        let name = source.file_name().unwrap().to_string_lossy();
        Ok(format!("asset-{name}"))
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn should_create_a_vanilla_instance_when_given_valid_params() {
    let (_temp, location_info) = setup_temp_location();
    let instances_root = location_info.instances_dir();

    let storage = Arc::new(MemInstanceStorage::new());
    let loader_service = Arc::new(MockLoaderService {
        try_get_default_result: None,
    });
    let install_service = Arc::new(NoOpInstallService);
    let file_service = Arc::new(MemInstanceFileService::new(instances_root));
    let watcher = Arc::new(NoOpWatcher);
    let emitter: SharedEventEmitter = Arc::new(NoOpEmitter);

    let use_case = make_use_case(
        storage.clone(),
        loader_service,
        install_service,
        file_service,
        watcher,
        emitter,
    );

    let new_instance = NewInstance {
        name: "Test Instance".to_string(),
        game_version: "1.21".to_string(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    let instance_id = use_case
        .execute(new_instance)
        .await
        .expect("Create vanilla instance should succeed");

    // Verify instance was stored
    let stored = storage
        .get(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .expect("Instance should exist");
    assert_eq!(stored.name(), "Test Instance");
    assert_eq!(stored.game_version, "1.21");
    assert_eq!(stored.loader, ModLoader::Vanilla);

    // Verify instance directory was created (via LocationInfo naming convention)
    let instance_dir =
        location_info.instance_dir(&InstanceId::try_from(instance_id.as_str()).unwrap());
    assert!(
        instance_dir.exists(),
        "Instance directory should exist on disk"
    );
}

#[tokio::test]
async fn should_create_an_instance_when_given_a_loader_and_an_explicit_version() {
    let (_temp, location_info) = setup_temp_location();
    let instances_root = location_info.instances_dir();

    let storage = Arc::new(MemInstanceStorage::new());
    let loader_service = Arc::new(MockLoaderService {
        try_get_default_result: Some(LoaderVersionPreference::Exact("0.16.5".to_string())),
    });
    let install_service = Arc::new(NoOpInstallService);
    let file_service = Arc::new(MemInstanceFileService::new(instances_root));
    let watcher = Arc::new(NoOpWatcher);
    let emitter: SharedEventEmitter = Arc::new(NoOpEmitter);

    let use_case = make_use_case(
        storage.clone(),
        loader_service,
        install_service,
        file_service,
        watcher,
        emitter,
    );

    let new_instance = NewInstance {
        name: "Fabric Test".to_string(),
        game_version: "1.20.1".to_string(),
        mod_loader: ModLoader::Fabric,
        loader_version: Some(LoaderVersionPreference::Exact("0.16.5".to_string())),
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    let instance_id = use_case
        .execute(new_instance)
        .await
        .expect("Create Fabric instance should succeed");

    let stored = storage
        .get(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    assert_eq!(stored.loader, ModLoader::Fabric);
    assert_eq!(
        stored.loader_version,
        Some(LoaderVersionPreference::Exact("0.16.5".to_string()))
    );
}

#[tokio::test]
async fn should_pick_the_default_loader_version_when_none_is_given() {
    let (_temp, location_info) = setup_temp_location();
    let instances_root = location_info.instances_dir();

    let storage = Arc::new(MemInstanceStorage::new());
    let loader_service = Arc::new(MockLoaderService {
        try_get_default_result: Some(LoaderVersionPreference::Stable),
    });
    let install_service = Arc::new(NoOpInstallService);
    let file_service = Arc::new(MemInstanceFileService::new(instances_root));
    let watcher = Arc::new(NoOpWatcher);
    let emitter: SharedEventEmitter = Arc::new(NoOpEmitter);

    let use_case = make_use_case(
        storage.clone(),
        loader_service,
        install_service,
        file_service,
        watcher,
        emitter,
    );

    let new_instance = NewInstance {
        name: "Forge Test".to_string(),
        game_version: "1.20.1".to_string(),
        mod_loader: ModLoader::Forge,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    let instance_id = use_case
        .execute(new_instance)
        .await
        .expect("Create Forge instance should succeed");

    let stored = storage
        .get(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    assert_eq!(stored.loader, ModLoader::Forge);
    assert_eq!(stored.loader_version, Some(LoaderVersionPreference::Stable));
}

#[tokio::test]
async fn should_run_the_install_service_when_an_instance_is_created() {
    struct TrackedInstallService(Arc<std::sync::atomic::AtomicBool>);

    #[async_trait]
    impl InstanceInstallService for TrackedInstallService {
        async fn execute(
            &self,
            _instance_id: InstanceId,
            _force: bool,
        ) -> Result<(), InstanceError> {
            self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    let (_temp, location_info) = setup_temp_location();
    let instances_root = location_info.instances_dir();

    let storage = Arc::new(MemInstanceStorage::new());
    let loader_service = Arc::new(MockLoaderService {
        try_get_default_result: None,
    });

    // Use a shared flag to verify the install service was called
    let install_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let install_called_clone = install_called.clone();

    let file_service = Arc::new(MemInstanceFileService::new(instances_root));
    let watcher = Arc::new(NoOpWatcher);
    let emitter: SharedEventEmitter = Arc::new(NoOpEmitter);

    let use_case = make_use_case(
        storage.clone(),
        loader_service,
        Arc::new(TrackedInstallService(install_called_clone)),
        file_service,
        watcher,
        emitter,
    );

    let new_instance = NewInstance {
        name: "Install Test".to_string(),
        game_version: "1.21".to_string(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(false),
        pack_info: None,
    };

    use_case
        .execute(new_instance)
        .await
        .expect("Create instance should succeed");

    assert!(
        install_called.load(std::sync::atomic::Ordering::SeqCst),
        "Install service should have been called when skip_install_instance is false"
    );
}

#[tokio::test]
async fn should_skip_the_install_when_it_is_not_requested() {
    struct SkippedInstallService(Arc<std::sync::atomic::AtomicBool>);

    #[async_trait]
    impl InstanceInstallService for SkippedInstallService {
        async fn execute(
            &self,
            _instance_id: InstanceId,
            _force: bool,
        ) -> Result<(), InstanceError> {
            self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    let (_temp, location_info) = setup_temp_location();
    let instances_root = location_info.instances_dir();

    let storage = Arc::new(MemInstanceStorage::new());
    let loader_service = Arc::new(MockLoaderService {
        try_get_default_result: None,
    });

    let install_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let install_called_clone = install_called.clone();

    let file_service = Arc::new(MemInstanceFileService::new(instances_root));
    let watcher = Arc::new(NoOpWatcher);
    let emitter: SharedEventEmitter = Arc::new(NoOpEmitter);

    let use_case = make_use_case(
        storage.clone(),
        loader_service,
        Arc::new(SkippedInstallService(install_called_clone)),
        file_service,
        watcher,
        emitter,
    );

    let new_instance = NewInstance {
        name: "Skip Install".to_string(),
        game_version: "1.21".to_string(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    use_case
        .execute(new_instance)
        .await
        .expect("Create instance should succeed");

    assert!(
        !install_called.load(std::sync::atomic::Ordering::SeqCst),
        "Install service should NOT have been called when skip_install_instance is true"
    );
}

#[tokio::test]
async fn should_sanitize_the_name_when_it_has_unsafe_characters() {
    let (_temp, location_info) = setup_temp_location();
    let instances_root = location_info.instances_dir();

    let storage = Arc::new(MemInstanceStorage::new());
    let loader_service = Arc::new(MockLoaderService {
        try_get_default_result: None,
    });
    let install_service = Arc::new(NoOpInstallService);
    let file_service = Arc::new(MemInstanceFileService::new(instances_root));
    let watcher = Arc::new(NoOpWatcher);
    let emitter: SharedEventEmitter = Arc::new(NoOpEmitter);

    let use_case = make_use_case(
        storage.clone(),
        loader_service,
        install_service,
        file_service,
        watcher,
        emitter,
    );

    let new_instance = NewInstance {
        name: "Bad/Name:*Test?".to_string(),
        game_version: "1.21".to_string(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    let instance_id = use_case
        .execute(new_instance)
        .await
        .expect("Create instance with special chars should succeed");

    // The sanitized name is the instance id, should have underscores instead
    assert!(!instance_id.contains('/'), "ID should not contain slashes");
    assert!(!instance_id.contains(':'), "ID should not contain colons");
    assert!(
        !instance_id.contains('*'),
        "ID should not contain asterisks"
    );
}

#[tokio::test]
async fn should_roll_back_when_storage_fails() {
    // Storage that always fails on upsert
    struct FailingStorage;

    #[async_trait]
    impl InstanceStorage for FailingStorage {
        async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
            Ok(vec![])
        }
        async fn get(&self, _id: &InstanceId) -> Result<Instance, InstanceError> {
            Err(InstanceError::NotFound {
                instance_id: "x".to_string(),
            })
        }
        async fn upsert(&self, _instance: &Instance) -> Result<(), InstanceError> {
            Err(InstanceError::Storage("disk full".to_string()))
        }
        async fn remove(&self, _id: &InstanceId) -> Result<(), InstanceError> {
            Ok(())
        }
    }

    let (_temp, location_info) = setup_temp_location();
    let instances_root = location_info.instances_dir();

    let storage = Arc::new(FailingStorage);
    let loader_service = Arc::new(MockLoaderService {
        try_get_default_result: None,
    });
    let install_service = Arc::new(NoOpInstallService);
    let file_service = Arc::new(MemInstanceFileService::new(instances_root.clone()));
    let watcher = Arc::new(NoOpWatcher);
    let emitter: SharedEventEmitter = Arc::new(NoOpEmitter);

    let use_case = make_use_case(
        storage,
        loader_service,
        install_service,
        file_service,
        watcher,
        emitter,
    );

    let new_instance = NewInstance {
        name: "Rollback Test".to_string(),
        game_version: "1.21".to_string(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    let result = use_case.execute(new_instance).await;
    assert!(result.is_err(), "Should fail when storage upsert fails");

    // Verify the instance directory was cleaned up during rollback
    let dir = instances_root.join("Rollback_Test");
    assert!(
        !dir.exists(),
        "Instance directory should be cleaned up on rollback"
    );
}

#[tokio::test]
async fn should_store_the_asset_id_when_the_new_instance_has_an_icon() {
    let (temp, location_info) = setup_temp_location();
    let icon = temp.path().join("icon.png");
    std::fs::write(&icon, b"png").unwrap();

    let storage = Arc::new(MemInstanceStorage::new());
    let use_case = make_use_case(
        storage.clone(),
        Arc::new(MockLoaderService {
            try_get_default_result: None,
        }),
        Arc::new(NoOpInstallService),
        Arc::new(MemInstanceFileService::new(location_info.instances_dir())),
        Arc::new(NoOpWatcher),
        Arc::new(NoOpEmitter),
    );

    let instance_id = use_case
        .execute(NewInstance {
            name: "With Icon".to_string(),
            game_version: "1.21".to_string(),
            mod_loader: ModLoader::Vanilla,
            loader_version: None,
            icon_path: Some(icon.to_string_lossy().into_owned()),
            skip_install_instance: Some(true),
            pack_info: None,
        })
        .await
        .expect("an instance with an icon should be created");

    let stored = storage
        .get(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    assert_eq!(stored.icon_path(), Some("asset-icon.png"));
}

#[tokio::test]
async fn should_create_the_instance_without_an_icon_when_the_icon_cannot_be_imported() {
    let (temp, location_info) = setup_temp_location();

    let storage = Arc::new(MemInstanceStorage::new());
    let use_case = make_use_case(
        storage.clone(),
        Arc::new(MockLoaderService {
            try_get_default_result: None,
        }),
        Arc::new(NoOpInstallService),
        Arc::new(MemInstanceFileService::new(location_info.instances_dir())),
        Arc::new(NoOpWatcher),
        Arc::new(NoOpEmitter),
    );

    let instance_id = use_case
        .execute(NewInstance {
            name: "Missing Icon".to_string(),
            game_version: "1.21".to_string(),
            mod_loader: ModLoader::Vanilla,
            loader_version: None,
            icon_path: Some(
                temp.path()
                    .join("no-such-icon.png")
                    .to_string_lossy()
                    .into_owned(),
            ),
            skip_install_instance: Some(true),
            pack_info: None,
        })
        .await
        .expect("an unreadable icon must not fail the creation");

    let stored = storage
        .get(&InstanceId::try_from(instance_id.as_str()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        stored.icon_path(),
        None,
        "a host path must never be kept as the icon"
    );
}
