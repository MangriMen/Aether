use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    time::Duration,
};

use async_trait::async_trait;
use extism::{Function, Manifest, PluginBuilder, UserData, Wasm};
use tokio::sync::Notify;

use super::super::{PluginInfrastructureListener, extism::models::ExtismPluginInstance};
use crate::{
    features::{
        events::{Event, EventEmitter, EventError, PluginEvent},
        instance::{
            CapabilityMetadata, ContentProvider, Importer, ImporterCapabilityMetadata, Updater,
        },
        plugins::{
            AllowedPath, ApiConfig, DisablePluginUseCase, EditPluginSettingsUseCase,
            EnablePluginUseCase, FolderMounts, ForceEnablePluginUseCase, GetPluginDtoUseCase,
            GrantPluginFolderAccessUseCase, GrantPluginHostAccessUseCase, ListPluginsDtoUseCase,
            LoadConfig, LoadConfigType, Plugin, PluginCapabilities, PluginError, PluginHandle,
            PluginImporterCapability, PluginInternalEvent, PluginLoader, PluginLoaderRegistry,
            PluginManifest, PluginMetadata, PluginRegistry, PluginSettings, PluginSettingsStorage,
            PluginState, ReloadPluginUseCase, RuntimeConfig, core_folders, partition_mountable,
        },
        settings::{LocationInfo, Settings, SettingsError, SettingsStorage},
    },
    shared::capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
};

pub(super) const PLUGIN_ID: &str = "packwiz";
pub(super) const PATIENCE: Duration = Duration::from_secs(5);

/// `work` blocks in the host function `wait` until the test lets it go; `import` returns at once.
pub(super) const PLUGIN: &str = r#"(module
    (import "extism:host/user" "wait" (func $wait (result i64)))
    (func (export "work") (result i32)
        (drop (call $wait))
        (i32.const 0))
    (func (export "import") (result i32)
        (i32.const 0)))"#;

pub(super) struct NoOpEmitter;

#[async_trait]
impl EventEmitter<Event> for NoOpEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }
    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

pub(super) struct NoPluginSettings;

#[async_trait]
impl PluginSettingsStorage for NoPluginSettings {
    async fn get(&self, _plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        Ok(None)
    }

    async fn upsert(
        &self,
        _plugin_id: &str,
        _settings: &PluginSettings,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct MemorySettings(Mutex<Settings>);

#[async_trait]
impl SettingsStorage for MemorySettings {
    async fn get(&self) -> Result<Settings, SettingsError> {
        Ok(self.0.lock().unwrap().clone())
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        *self.0.lock().unwrap() = settings.clone();
        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
    ) -> Result<Settings, SettingsError> {
        let mut settings = self.0.lock().unwrap();
        let (updated, _) = f(settings.clone());
        *settings = updated.clone();
        Ok(updated)
    }
}

/// The long call: `wait` tells the test it has started, then blocks until let go.
pub(super) struct Gate {
    pub(super) started: Mutex<mpsc::Sender<()>>,
    pub(super) release: Mutex<mpsc::Receiver<()>>,
}

/// Builds instances of [`PLUGIN`] and ends them the way the Extism loader does. A load can be
/// held up until the test lets it go, or made to fail.
pub(super) struct WatLoader {
    pub(super) gate: Arc<Gate>,
    pub(super) loads: AtomicUsize,
    pub(super) unloads: AtomicUsize,
    pub(super) fail_next_load: AtomicBool,
    pub(super) hold_loads: AtomicBool,
    pub(super) load_started: Notify,
    pub(super) load_released: Notify,
    pub(super) location_info: Arc<LocationInfo>,
}

#[async_trait]
impl PluginLoader for WatLoader {
    async fn load(
        &self,
        manifest: &PluginManifest,
        settings: Option<&PluginSettings>,
    ) -> Result<PluginHandle, PluginError> {
        if self.hold_loads.load(Ordering::SeqCst) {
            self.load_started.notify_one();
            self.load_released.notified().await;
        }
        let plugin_id = manifest.metadata.id.clone();
        if self.fail_next_load.swap(false, Ordering::SeqCst) {
            return Err(PluginError::LoadFailed {
                plugin_id,
                reason: "the wasm does not compile".into(),
            });
        }

        let gate = Arc::clone(&self.gate);
        let wait = Function::new(
            "wait",
            [],
            [extism::ValType::I64],
            UserData::new(()),
            move |_, _, outputs, _| {
                gate.started.lock().unwrap().send(()).unwrap();
                gate.release
                    .lock()
                    .unwrap()
                    .recv_timeout(PATIENCE)
                    .expect("the test lets the call go");
                outputs[0] = extism::Val::I64(0);
                Ok(())
            },
        );
        let plugin = PluginBuilder::new(Manifest::new([Wasm::data(PLUGIN.as_bytes().to_vec())]))
            .with_functions([wait])
            .build()
            .expect("the test plugin should load");

        // Mounts what the Extism loader does.
        let folders = FolderMounts::of(
            partition_mountable(
                settings.map_or(&[][..], |settings| settings.allowed_paths.as_slice()),
                &core_folders(&self.location_info, &plugin_id),
            )
            .0,
        );

        self.loads.fetch_add(1, Ordering::SeqCst);
        Ok(PluginHandle::new(ExtismPluginInstance::new(plugin, plugin_id)).with_folders(folders))
    }

    async fn unload(&self, instance: PluginHandle) -> Result<(), PluginError> {
        instance
            .retire(|plugin| plugin.handle_event(&PluginInternalEvent::Unloaded))
            .await?
            .ok();
        self.unloads.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn host_functions(&self) -> Vec<&'static str> {
        vec![]
    }
}

pub(super) type Listener = PluginInfrastructureListener<
    MemoryCapabilityRegistry<Arc<dyn Importer>>,
    MemoryCapabilityRegistry<Arc<dyn Updater>>,
    MemoryCapabilityRegistry<Arc<dyn ContentProvider>>,
>;

pub(super) struct Harness {
    pub(super) plugins: Arc<PluginRegistry>,
    pub(super) loaders: Arc<PluginLoaderRegistry>,
    pub(super) settings: Arc<MemorySettings>,
    pub(super) loader: Arc<WatLoader>,
    pub(super) started: Mutex<mpsc::Receiver<()>>,
    pub(super) release: Mutex<mpsc::Sender<()>>,
    pub(super) importers: Arc<MemoryCapabilityRegistry<Arc<dyn Importer>>>,
    pub(super) listener: Listener,
    pub(super) location_info: Arc<LocationInfo>,
    pub(super) _root: tempfile::TempDir,
}

pub(super) fn manifest() -> PluginManifest {
    PluginManifest {
        metadata: PluginMetadata {
            id: PLUGIN_ID.into(),
            name: "Packwiz".into(),
            version: semver::Version::new(0, 1, 0),
            description: None,
            authors: vec![],
            license: None,
        },
        runtime: RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths: vec![],
        },
        load: LoadConfig::Extism {
            file: "plugin.wasm".into(),
            memory_limit: None,
        },
        api: ApiConfig {
            version: semver::VersionReq::STAR,
            features: vec![],
            host_functions: Some(vec![]),
        },
    }
}

pub(super) fn importer_capability() -> PluginImporterCapability {
    PluginImporterCapability {
        metadata: ImporterCapabilityMetadata {
            base: CapabilityMetadata {
                id: "packwiz".into(),
                name: "Packwiz".into(),
                description: None,
                icon: None,
            },
            field_label: None,
            supported_extensions: Vec::new(),
        },
        handler: "import".into(),
    }
}

impl Harness {
    /// The plugin enabled and loaded, its importer registered.
    pub(super) async fn enabled() -> Self {
        let plugins = Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter)));
        plugins.insert(
            PLUGIN_ID.to_owned(),
            Plugin {
                manifest: manifest(),
                capabilities: Some(PluginCapabilities {
                    importers: vec![importer_capability()],
                    ..PluginCapabilities::default()
                }),
                hash: "hash".into(),
                verified: false,
                state: PluginState::NotLoaded,
            },
        );

        let root = tempfile::tempdir().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            root.path().to_path_buf(),
            root.path().to_path_buf(),
        ));
        let (started_tx, started) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let loader = Arc::new(WatLoader {
            gate: Arc::new(Gate {
                started: Mutex::new(started_tx),
                release: Mutex::new(release_rx),
            }),
            loads: AtomicUsize::new(0),
            unloads: AtomicUsize::new(0),
            fail_next_load: AtomicBool::new(false),
            hold_loads: AtomicBool::new(false),
            load_started: Notify::new(),
            load_released: Notify::new(),
            location_info: Arc::clone(&location_info),
        });
        let loaders = Arc::new(PluginLoaderRegistry::new(HashMap::from([(
            LoadConfigType::Extism,
            Arc::clone(&loader) as Arc<dyn PluginLoader>,
        )])));

        let importers = Arc::new(MemoryCapabilityRegistry::<Arc<dyn Importer>>::default());
        let listener = PluginInfrastructureListener::new(
            Arc::clone(&plugins),
            Arc::clone(&importers),
            Arc::new(MemoryCapabilityRegistry::default()),
            Arc::new(MemoryCapabilityRegistry::default()),
            Arc::clone(&location_info),
        );

        let harness = Self {
            plugins,
            loaders,
            settings: Arc::default(),
            loader,
            started: Mutex::new(started),
            release: Mutex::new(release),
            importers,
            listener,
            location_info,
            _root: root,
        };
        EnablePluginUseCase::new(
            Arc::clone(&harness.plugins),
            Arc::clone(&harness.loaders),
            Arc::new(NoPluginSettings),
            harness.settings.clone(),
        )
        .execute(PLUGIN_ID.into())
        .await
        .expect("the plugin should enable");
        harness.sync_registries().await;
        harness
    }

    pub(super) fn reload(&self) -> ReloadPluginUseCase {
        ReloadPluginUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(&self.loaders),
            Arc::new(NoPluginSettings),
            self.settings.clone(),
        )
    }

    pub(super) fn disable(&self) -> DisablePluginUseCase {
        DisablePluginUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(&self.loaders),
            self.settings.clone(),
        )
    }

    /// What the listener does on every state change; run by hand, as the events go nowhere here.
    pub(super) async fn sync_registries(&self) {
        self.listener
            .on_plugin_event(PluginEvent::Edit {
                plugin_id: PLUGIN_ID.into(),
            })
            .await;
    }

    pub(super) fn let_go(&self) {
        self.release.lock().unwrap().send(()).unwrap();
    }

    pub(super) fn state(&self) -> PluginState {
        self.plugins.get(PLUGIN_ID).unwrap().state.clone()
    }

    pub(super) fn handle(&self) -> PluginHandle {
        match self.state() {
            PluginState::Loaded(handle) => handle,
            _ => panic!("the plugin should be loaded"),
        }
    }

    pub(super) fn is_enabled(&self) -> bool {
        self.settings
            .0
            .lock()
            .unwrap()
            .enabled_plugins()
            .contains(PLUGIN_ID)
    }

    pub(super) async fn importer(&self) -> Option<Arc<dyn Importer>> {
        self.importers
            .find_by_plugin_and_capability_id(PLUGIN_ID, "packwiz")
            .await
            .ok()
            .map(|entry| entry.capability)
    }

    /// Starts `work` on `handle` and returns once the plugin is inside it.
    pub(super) async fn start_long_call(
        &self,
        handle: &PluginHandle,
    ) -> tokio::task::JoinHandle<Result<(), PluginError>> {
        let call = tokio::spawn({
            let handle = handle.clone();
            async move {
                handle
                    .run("work", |plugin| plugin.call_bytes("work", &[]).map(|_| ()))
                    .await
                    .flatten()
            }
        });
        tokio::task::block_in_place(|| self.started.lock().unwrap().recv_timeout(PATIENCE))
            .expect("the call starts");
        call
    }
}

pub(super) async fn noop(handle: &PluginHandle) -> Result<(), PluginError> {
    handle
        .run("import", |plugin| {
            plugin.call_bytes("import", &[]).map(|_| ())
        })
        .await
        .flatten()
}

pub(super) fn is_stopped(result: &Result<(), PluginError>) -> bool {
    matches!(result, Err(PluginError::Stopped { plugin_id }) if plugin_id == PLUGIN_ID)
}

impl Harness {
    pub(super) fn enable(&self) -> EnablePluginUseCase {
        EnablePluginUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(&self.loaders),
            Arc::new(NoPluginSettings),
            self.settings.clone(),
        )
    }

    pub(super) fn force_enable(&self) -> ForceEnablePluginUseCase {
        ForceEnablePluginUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(&self.loaders),
            Arc::new(NoPluginSettings),
            self.settings.clone(),
        )
    }
}

#[derive(Default)]
pub(super) struct MemoryPluginSettings(pub(super) Mutex<Option<PluginSettings>>);

#[async_trait]
impl PluginSettingsStorage for MemoryPluginSettings {
    async fn get(&self, _plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        Ok(self.0.lock().unwrap().clone())
    }

    async fn upsert(&self, _plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError> {
        *self.0.lock().unwrap() = Some(settings.clone());
        Ok(())
    }
}

impl MemoryPluginSettings {
    pub(super) fn folders(&self) -> Vec<AllowedPath> {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .map(|settings| settings.allowed_paths.clone())
            .unwrap_or_default()
    }
}

impl Harness {
    pub(super) fn grant(
        &self,
        settings: &Arc<MemoryPluginSettings>,
    ) -> GrantPluginFolderAccessUseCase {
        GrantPluginFolderAccessUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(settings) as Arc<dyn PluginSettingsStorage>,
            Arc::new(self.reload_from(settings)),
            Arc::clone(&self.location_info),
        )
    }
}

impl MemoryPluginSettings {
    pub(super) fn hosts(&self) -> Vec<String> {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .map(|settings| settings.allowed_hosts.clone())
            .unwrap_or_default()
    }
}

impl Harness {
    pub(super) fn grant_host(
        &self,
        settings: &Arc<MemoryPluginSettings>,
    ) -> GrantPluginHostAccessUseCase {
        GrantPluginHostAccessUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(settings) as Arc<dyn PluginSettingsStorage>,
        )
    }
}

impl Harness {
    /// A restart that builds the instance from `settings`, as the real one does from the disk.
    pub(super) fn reload_from(&self, settings: &Arc<MemoryPluginSettings>) -> ReloadPluginUseCase {
        ReloadPluginUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(&self.loaders),
            Arc::clone(settings) as Arc<dyn PluginSettingsStorage>,
            self.settings.clone(),
        )
    }

    pub(super) fn edit(&self, settings: &Arc<MemoryPluginSettings>) -> EditPluginSettingsUseCase {
        EditPluginSettingsUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(settings) as Arc<dyn PluginSettingsStorage>,
            Arc::clone(&self.location_info),
        )
    }

    pub(super) async fn restart_needed(&self, settings: &Arc<MemoryPluginSettings>) -> bool {
        let dto = GetPluginDtoUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(settings) as Arc<dyn PluginSettingsStorage>,
        )
        .execute(PLUGIN_ID.into())
        .await
        .unwrap();
        let listed = ListPluginsDtoUseCase::new(
            Arc::clone(&self.plugins),
            Arc::clone(settings) as Arc<dyn PluginSettingsStorage>,
        )
        .execute()
        .await
        .unwrap();
        assert_eq!(listed[0].restart_needed, dto.restart_needed);
        dto.restart_needed
    }
}

/// Hands out what it holds, then takes its time before returning it: two changes that were not
/// made one after the other both read the settings before either saves.
#[derive(Default)]
pub(super) struct SlowPluginSettings(pub(super) MemoryPluginSettings);

#[async_trait]
impl PluginSettingsStorage for SlowPluginSettings {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        let settings = self.0.get(plugin_id).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        settings
    }

    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError> {
        self.0.upsert(plugin_id, settings).await
    }
}
