use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::{
    events::{Event, EventEmitter, EventError},
    plugins::{
        ApiConfig, EnablePluginUseCase, ForceEnablePluginUseCase, GrantPluginHostAccessUseCase,
        LoadConfig, LoadConfigType, ManifestError, PLUGIN_API_VERSION, PathMapping, Plugin,
        PluginError, PluginHandle, PluginInstance, PluginInternalEvent, PluginLoader,
        PluginLoaderRegistry, PluginManifest, PluginMetadata, PluginRegistry, PluginSettings,
        PluginSettingsStorage, PluginState, PluginStateReason, RuntimeConfig,
    },
    settings::{Settings, SettingsError, SettingsStorage},
};

// ─── Inline mocks ──────────────────────────────────────────────

struct NoOpEmitter;

#[async_trait]
impl EventEmitter<Event> for NoOpEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }
    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

#[derive(Default)]
struct MockPluginSettingsStorage {
    settings: Mutex<HashMap<String, PluginSettings>>,
}

#[async_trait]
impl PluginSettingsStorage for MockPluginSettingsStorage {
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSettings>, PluginError> {
        Ok(self
            .settings
            .lock()
            .unwrap()
            .get(plugin_id.as_str())
            .cloned())
    }

    async fn upsert(
        &self,
        plugin_id: &PluginId,
        settings: &PluginSettings,
    ) -> Result<(), PluginError> {
        self.settings
            .lock()
            .unwrap()
            .insert(plugin_id.to_string(), settings.clone());
        Ok(())
    }
}

struct MockSettingsStorage {
    settings: Mutex<Settings>,
}

impl Default for MockSettingsStorage {
    fn default() -> Self {
        Self {
            settings: Mutex::new(Settings::default()),
        }
    }
}

#[async_trait]
impl SettingsStorage for MockSettingsStorage {
    async fn get(&self) -> Result<Settings, SettingsError> {
        Ok(self.settings.lock().unwrap().clone())
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        *self.settings.lock().unwrap() = settings.clone();
        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
    ) -> Result<Settings, SettingsError> {
        let mut guard = self.settings.lock().unwrap();
        let (updated, _changed) = f(guard.clone());
        *guard = updated.clone();
        Ok(updated)
    }
}

struct MockPluginInstance;

impl PluginInstance for MockPluginInstance {
    fn get_id(&self) -> PluginId {
        PluginId::from_static("mock")
    }
    fn supports(&self, _name: &str) -> bool {
        false
    }
    fn call_bytes<'b>(&'b mut self, _name: &str, _args: &[u8]) -> Result<&'b [u8], PluginError> {
        Ok(&[])
    }
    fn handle_event(&mut self, _event: &PluginInternalEvent) -> Result<(), PluginError> {
        Ok(())
    }
}

struct AlwaysSucceedsLoader;

#[async_trait]
impl PluginLoader for AlwaysSucceedsLoader {
    async fn load(
        &self,
        _plugin: &PluginManifest,
        _settings: Option<&PluginSettings>,
    ) -> Result<PluginHandle, PluginError> {
        Ok(PluginHandle::new(MockPluginInstance))
    }

    async fn unload(&self, _instance: PluginHandle) -> Result<(), PluginError> {
        Ok(())
    }

    fn host_functions(&self) -> Vec<&'static str> {
        vec!["log", "get_id", "http_get"]
    }
}

// ─── Fixtures ──────────────────────────────────────────────────

fn manifest_with(
    allowed_paths: Vec<PathMapping>,
    api_version: semver::VersionReq,
) -> PluginManifest {
    PluginManifest {
        metadata: PluginMetadata {
            id: PluginId::from_static("test-plugin"),
            name: "Test Plugin".into(),
            version: semver::Version::new(0, 1, 0),
            description: None,
            authors: vec![],
            license: None,
        },
        runtime: RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths,
        },
        load: LoadConfig::Extism {
            file: "plugin.wasm".try_into().unwrap(),
            memory_limit: None,
        },
        api: ApiConfig {
            version: api_version,
            features: vec![],
            host_functions: Some(vec![]),
        },
    }
}

fn manifest_declaring(host_functions: &[&str]) -> PluginManifest {
    let mut manifest = manifest_with(vec![], semver::VersionReq::STAR);
    manifest.api.host_functions =
        Some(host_functions.iter().map(|&name| name.to_owned()).collect());
    manifest
}

fn absolute_path_mapping() -> PathMapping {
    let host_path = if cfg!(windows) {
        r"C:\absolute\path".to_owned()
    } else {
        "/absolute/path".to_owned()
    };
    PathMapping(host_path, "virtual".into())
}

struct Harness {
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry>,
    plugin_settings_storage: Arc<MockPluginSettingsStorage>,
    settings_storage: Arc<MockSettingsStorage>,
}

impl Harness {
    fn new(manifest: PluginManifest) -> Self {
        let plugin_registry = Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter)));
        plugin_registry.insert(
            manifest.metadata.id.clone(),
            Plugin {
                manifest,
                capabilities: None,
                hash: "hash".into(),
                verified: false,
                state: PluginState::NotLoaded,
            },
        );

        let mut loaders = HashMap::new();
        loaders.insert(
            LoadConfigType::Extism,
            Arc::new(AlwaysSucceedsLoader) as Arc<dyn PluginLoader>,
        );

        Self {
            plugin_registry,
            plugin_loader_registry: Arc::new(PluginLoaderRegistry::new(loaders)),
            plugin_settings_storage: Arc::new(MockPluginSettingsStorage::default()),
            settings_storage: Arc::new(MockSettingsStorage::default()),
        }
    }

    fn enable_use_case(&self) -> EnablePluginUseCase {
        EnablePluginUseCase::new(
            self.plugin_registry.clone(),
            self.plugin_loader_registry.clone(),
            self.plugin_settings_storage.clone(),
            self.settings_storage.clone(),
        )
    }

    fn force_enable_use_case(&self) -> ForceEnablePluginUseCase {
        ForceEnablePluginUseCase::new(
            self.plugin_registry.clone(),
            self.plugin_loader_registry.clone(),
            self.plugin_settings_storage.clone(),
            self.settings_storage.clone(),
        )
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn should_reject_enable_when_allowed_paths_are_absolute() {
    let manifest = manifest_with(vec![absolute_path_mapping()], semver::VersionReq::STAR);
    let harness = Harness::new(manifest);

    let result = harness
        .enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Manifest(ManifestError::InvalidPathMapping))
    ));
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Incompatible(_)));
}

#[tokio::test]
async fn should_reject_force_enable_when_allowed_paths_are_absolute_despite_an_old_api() {
    // API version requirement is unsatisfiable, proving that the manifest's
    // absolute allowed_paths violation is what actually blocks force-enable —
    // not the (bypassed) API version check.
    let manifest = manifest_with(
        vec![absolute_path_mapping()],
        semver::VersionReq::parse("=99999.0.0").unwrap(),
    );
    let harness = Harness::new(manifest);

    let result = harness
        .force_enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Manifest(ManifestError::InvalidPathMapping))
    ));
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Incompatible(_)));
}

#[tokio::test]
async fn should_mark_the_plugin_incompatible_with_both_versions_when_the_api_version_does_not_match()
 {
    let required = semver::VersionReq::parse("=99999.0.0").unwrap();
    let harness = Harness::new(manifest_with(vec![], required.clone()));

    let result = harness
        .enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(
        matches!(result, Err(PluginError::IncompatibleApiVersion { .. })),
        "{result:?}"
    );
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    let PluginState::Incompatible(reason) = state else {
        panic!("the plugin should be marked incompatible");
    };
    assert_eq!(
        reason,
        PluginStateReason::IncompatibleApiVersion {
            required,
            host: PLUGIN_API_VERSION.clone(),
        }
    );
}

#[tokio::test]
async fn should_bypass_the_api_version_check_when_force_enabling() {
    // Relative paths (valid) + unsatisfiable API version requirement: force-enable
    // must succeed by skipping only the version check.
    let manifest = manifest_with(
        vec![PathMapping("relative/path".into(), "virtual".into())],
        semver::VersionReq::parse("=99999.0.0").unwrap(),
    );
    let harness = Harness::new(manifest);

    let result = harness
        .force_enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(result.is_ok());
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Loaded(_)));
}

#[tokio::test]
async fn should_enable_when_paths_are_relative_and_the_api_is_compatible() {
    // Generic case: a non-empty relative allowed_paths entry, and an API version
    // requirement the host trivially satisfies.
    let manifest = manifest_with(
        vec![PathMapping("relative/path".into(), "virtual".into())],
        semver::VersionReq::STAR,
    );
    let harness = Harness::new(manifest);

    let result = harness
        .enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(result.is_ok());
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Loaded(_)));
}

#[tokio::test]
async fn should_enable_when_the_manifest_has_no_allowed_paths() {
    // A plugin that declares the host's own API version and no allowed_paths: the unconditional
    // `runtime.validate()` must let an empty list through. The version is the host's, not a
    // plugin's: this guards the host's behavior, so it holds across API bumps.
    let manifest = manifest_with(
        vec![],
        semver::VersionReq::parse(&PLUGIN_API_VERSION.to_string()).unwrap(),
    );
    let harness = Harness::new(manifest);

    let result = harness
        .enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(result.is_ok());
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Loaded(_)));
}

// ─── api.hostFunctions ─────────────────────────────────────────

#[tokio::test]
async fn should_reject_enable_when_the_loader_does_not_provide_a_declared_host_function() {
    let harness = Harness::new(manifest_declaring(&["log", "spawn_shell"]));

    let result = harness
        .enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    match result {
        Err(PluginError::Manifest(ManifestError::UnknownHostFunctions(names))) => {
            assert_eq!(names, ["spawn_shell"]);
        }
        other => panic!("expected UnknownHostFunctions, got: {other:?}"),
    }
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    let PluginState::Incompatible(PluginStateReason::InvalidManifest { details }) = state else {
        panic!("the plugin should be marked incompatible for its manifest");
    };
    assert!(details.contains("`spawn_shell`"), "{details}");
}

/// Force-enable overrides the API version check only: it cannot conjure up a host function
/// the launcher does not have.
#[tokio::test]
async fn should_reject_force_enable_when_the_loader_does_not_provide_a_declared_host_function() {
    let harness = Harness::new(manifest_declaring(&["spawn_shell"]));

    let result = harness
        .force_enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Manifest(ManifestError::UnknownHostFunctions(
            _
        )))
    ));
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Incompatible(_)));
}

#[tokio::test]
async fn should_enable_when_every_declared_host_function_is_provided() {
    let harness = Harness::new(manifest_declaring(&["log", "http_get"]));

    let result = harness
        .enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(result.is_ok(), "{result:?}");
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Loaded(_)));
}

/// A manifest without `api.hostFunctions` is incompatible, not granted every host function.
#[tokio::test]
async fn should_reject_enable_when_the_manifest_has_no_host_function_declaration() {
    let mut manifest = manifest_with(vec![], semver::VersionReq::STAR);
    manifest.api.host_functions = None;
    let harness = Harness::new(manifest);

    let result = harness
        .enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Manifest(
            ManifestError::HostFunctionsNotDeclared
        ))
    ));
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    let PluginState::Incompatible(PluginStateReason::InvalidManifest { details }) = state else {
        panic!("the plugin should be marked incompatible for its manifest");
    };
    assert!(details.contains("api.hostFunctions"), "{details}");
}

#[tokio::test]
async fn should_reject_force_enable_when_the_manifest_has_no_host_function_declaration() {
    let mut manifest = manifest_with(vec![], semver::VersionReq::STAR);
    manifest.api.host_functions = None;
    let harness = Harness::new(manifest);

    let result = harness
        .force_enable_use_case()
        .execute(PluginId::from_static("test-plugin"))
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Manifest(
            ManifestError::HostFunctionsNotDeclared
        ))
    ));
}

/// Forcing skips the API version check, not the plugin's state: a plugin already loaded is not
/// loaded a second time.
#[tokio::test]
async fn should_refuse_force_enable_when_the_plugin_is_already_loaded() {
    let manifest = manifest_with(
        vec![PathMapping("relative/path".into(), "virtual".into())],
        semver::VersionReq::parse("=99999.0.0").unwrap(),
    );
    let harness = Harness::new(manifest);
    let use_case = harness.force_enable_use_case();
    use_case
        .execute(PluginId::from_static("test-plugin"))
        .await
        .unwrap();

    let result = use_case.execute(PluginId::from_static("test-plugin")).await;

    assert!(
        matches!(result, Err(PluginError::AlreadyLoaded { .. })),
        "{result:?}"
    );
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest(&PluginId::from_static("test-plugin"))
        .unwrap();
    assert!(matches!(state, PluginState::Loaded(_)));
}

// ─── Settings changed at the same time ─────────────────────────

/// Hands out what it holds, then takes its time before returning it: two changes that were not
/// made one after the other both read the settings before either saves.
#[derive(Default)]
struct SlowPluginSettingsStorage {
    inner: MockPluginSettingsStorage,
    /// Notified when a read has started, so a test can start its second change in the middle of it.
    read_started: tokio::sync::Notify,
}

#[async_trait]
impl PluginSettingsStorage for SlowPluginSettingsStorage {
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSettings>, PluginError> {
        self.read_started.notify_one();
        let settings = self.inner.get(plugin_id).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        settings
    }

    async fn upsert(
        &self,
        plugin_id: &PluginId,
        settings: &PluginSettings,
    ) -> Result<(), PluginError> {
        self.inner.upsert(plugin_id, settings).await
    }
}

/// The grant starts while the force-enable is loading the plugin, so that without the settings
/// lock it reads the settings before the force-enable flag is saved and saves after it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_keep_a_host_granted_when_it_is_granted_during_a_force_enable() {
    let harness = Harness::new(manifest_with(
        vec![],
        semver::VersionReq::parse("=99999.0.0").unwrap(),
    ));
    let settings = Arc::new(SlowPluginSettingsStorage::default());
    let force_enable = ForceEnablePluginUseCase::new(
        harness.plugin_registry.clone(),
        harness.plugin_loader_registry.clone(),
        settings.clone(),
        harness.settings_storage.clone(),
    );
    let grant =
        GrantPluginHostAccessUseCase::new(harness.plugin_registry.clone(), settings.clone());

    let (forced, granted) = tokio::join!(
        force_enable.execute(PluginId::from_static("test-plugin")),
        async {
            settings.read_started.notified().await;
            grant
                .execute(
                    PluginId::from_static("test-plugin"),
                    "https://example.com/pack.toml".into(),
                    false,
                )
                .await
        }
    );

    forced.unwrap();
    granted.unwrap();
    let saved = settings
        .inner
        .get(&PluginId::from_static("test-plugin"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.allowed_hosts, ["example.com"]);
    assert_eq!(
        saved.force_enabled_at_api_version,
        Some(PLUGIN_API_VERSION.to_string())
    );
}
