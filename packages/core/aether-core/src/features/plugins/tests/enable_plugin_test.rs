use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::features::{
    events::{Event, EventEmitter, EventError},
    plugins::{
        ApiConfig, EnablePluginUseCase, ForceEnablePluginUseCase, LoadConfig, LoadConfigType,
        ManifestError, PathMapping, Plugin, PluginError, PluginInstance, PluginInternalEvent,
        PluginLoader, PluginLoaderRegistry, PluginManifest, PluginMetadata, PluginRegistry,
        PluginSettings, PluginSettingsStorage, PluginState, RuntimeConfig,
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
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        Ok(self.settings.lock().unwrap().get(plugin_id).cloned())
    }

    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError> {
        self.settings
            .lock()
            .unwrap()
            .insert(plugin_id.to_owned(), settings.clone());
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
    fn get_id(&self) -> String {
        "mock".to_owned()
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
    ) -> Result<Arc<tokio::sync::Mutex<dyn PluginInstance>>, PluginError> {
        Ok(Arc::new(tokio::sync::Mutex::new(MockPluginInstance)))
    }

    async fn unload(
        &self,
        _instance: Arc<tokio::sync::Mutex<dyn PluginInstance>>,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

// ─── Fixtures ──────────────────────────────────────────────────

fn manifest_with(
    allowed_paths: Vec<PathMapping>,
    api_version: semver::VersionReq,
) -> PluginManifest {
    PluginManifest {
        metadata: PluginMetadata {
            id: "test-plugin".into(),
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
            file: "plugin.wasm".into(),
            memory_limit: None,
        },
        api: ApiConfig {
            version: api_version,
            features: vec![],
        },
    }
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
async fn enable_rejects_absolute_allowed_paths() {
    let manifest = manifest_with(vec![absolute_path_mapping()], semver::VersionReq::STAR);
    let harness = Harness::new(manifest);

    let result = harness
        .enable_use_case()
        .execute("test-plugin".into())
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Manifest(ManifestError::InvalidPathMapping))
    ));
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest("test-plugin")
        .unwrap();
    assert!(matches!(state, PluginState::Incompatible(_)));
}

#[tokio::test]
async fn force_enable_rejects_absolute_allowed_paths_even_with_incompatible_api() {
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
        .execute("test-plugin".into())
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Manifest(ManifestError::InvalidPathMapping))
    ));
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest("test-plugin")
        .unwrap();
    assert!(matches!(state, PluginState::Incompatible(_)));
}

#[tokio::test]
async fn force_enable_still_bypasses_api_version_check() {
    // Relative paths (valid) + unsatisfiable API version requirement: force-enable
    // must succeed by skipping only the version check.
    let manifest = manifest_with(
        vec![PathMapping("relative/path".into(), "virtual".into())],
        semver::VersionReq::parse("=99999.0.0").unwrap(),
    );
    let harness = Harness::new(manifest);

    let result = harness
        .force_enable_use_case()
        .execute("test-plugin".into())
        .await;

    assert!(result.is_ok());
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest("test-plugin")
        .unwrap();
    assert!(matches!(state, PluginState::Loaded(_)));
}

#[tokio::test]
async fn enable_succeeds_with_relative_paths_and_compatible_api() {
    // Mirrors packwiz: relative allowed_paths, API version requirement the host satisfies.
    let manifest = manifest_with(
        vec![PathMapping("relative/path".into(), "virtual".into())],
        semver::VersionReq::STAR,
    );
    let harness = Harness::new(manifest);

    let result = harness
        .enable_use_case()
        .execute("test-plugin".into())
        .await;

    assert!(result.is_ok());
    let (state, _) = harness
        .plugin_registry
        .get_state_and_manifest("test-plugin")
        .unwrap();
    assert!(matches!(state, PluginState::Loaded(_)));
}
