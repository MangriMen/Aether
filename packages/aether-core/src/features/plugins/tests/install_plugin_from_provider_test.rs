use std::{
    io::{Cursor, Write},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;
use tempfile::TempDir;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::features::instance::PluginId;
use crate::features::{
    events::{Event, EventEmitter, EventError},
    plugins::{
        InstallPluginFromProviderUseCase, PluginError, PluginProvider, PluginProviderError,
        PluginProviderFactory, PluginRegistry, PluginSettings, PluginSettingsStorage, PluginSource,
        PluginSourceStorage, PluginSourceType, PluginSyncService, PluginVerification,
        PluginVerificationStorage, ProviderPluginPreview, ProviderRelease, ProviderUpdateInfo,
        infra::{
            FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
            FsPluginVerificationStorage, ZipPluginExtractor,
        },
    },
    settings::LocationInfo,
};

const PLUGIN_ID: &str = "test-plugin";

fn plugin_id() -> PluginId {
    PluginId::from_static(PLUGIN_ID)
}

const MANIFEST: &str = r#"{
    "metadata": {
        "id": "test-plugin",
        "name": "Test Plugin",
        "version": "0.2.0",
        "description": null,
        "authors": [],
        "license": null
    },
    "runtime": { "allowedHosts": [], "allowedPaths": [] },
    "load": { "type": "extism", "file": "plugin.wasm", "memoryLimit": null },
    "api": { "version": "*", "features": [] }
}"#;

fn plugin_zip(wasm: &[u8]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in [
        ("manifest.json", MANIFEST.as_bytes()),
        ("plugin.wasm", wasm),
    ] {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// A GitHub-like source whose download is `archive`.
struct Download {
    archive: Vec<u8>,
}

#[async_trait]
impl PluginProvider for Download {
    async fn fetch_preview(&self, _: &str) -> Result<ProviderPluginPreview, PluginProviderError> {
        unimplemented!("the install does not preview")
    }

    async fn fetch_latest_version(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<ProviderUpdateInfo, PluginProviderError> {
        unimplemented!("the install does not look for updates")
    }

    async fn download_plugin(&self, _: &str) -> Result<Vec<u8>, PluginProviderError> {
        Ok(self.archive.clone())
    }

    fn source_type(&self) -> PluginSourceType {
        PluginSourceType::GitHub
    }

    fn parse_identifier(&self, raw: &str) -> Result<String, PluginProviderError> {
        Ok(raw.to_owned())
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

#[derive(Default)]
struct CountingSync(AtomicUsize);

#[async_trait]
impl PluginSyncService for CountingSync {
    async fn execute(&self) -> Result<(), PluginError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn disk_full() -> PluginError {
    PluginError::Storage(std::io::Error::other("disk full").into())
}

struct FailingSourceSave(Arc<FsPluginSourceStorage>);

#[async_trait]
impl PluginSourceStorage for FailingSourceSave {
    async fn save(&self, _: &PluginId, _: &PluginSource) -> Result<(), PluginError> {
        Err(disk_full())
    }
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSource>, PluginError> {
        self.0.get(plugin_id).await
    }
    async fn remove(&self, plugin_id: &PluginId) -> Result<(), PluginError> {
        self.0.remove(plugin_id).await
    }
}

struct FailingVerificationSave(Arc<FsPluginVerificationStorage>);

#[async_trait]
impl PluginVerificationStorage for FailingVerificationSave {
    async fn save(&self, _: &PluginId, _: &PluginVerification) -> Result<(), PluginError> {
        Err(disk_full())
    }
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginVerification>, PluginError> {
        self.0.get(plugin_id).await
    }
    async fn remove(&self, plugin_id: &PluginId) -> Result<(), PluginError> {
        self.0.remove(plugin_id).await
    }
}

struct Fixture {
    _temp: TempDir,
    location_info: Arc<LocationInfo>,
    registry: Arc<PluginRegistry>,
    settings: Arc<FsPluginSettingsStorage>,
    source: Arc<FsPluginSourceStorage>,
    verification: Arc<FsPluginVerificationStorage>,
    sync: Arc<CountingSync>,
}

impl Fixture {
    fn empty() -> Self {
        let temp = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().join("config"),
        ));
        Self {
            registry: Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter))),
            settings: Arc::new(FsPluginSettingsStorage::new(location_info.clone())),
            source: Arc::new(FsPluginSourceStorage::new(location_info.clone())),
            verification: Arc::new(FsPluginVerificationStorage::new(location_info.clone())),
            sync: Arc::default(),
            location_info,
            _temp: temp,
        }
    }

    async fn install(&self, archive: Vec<u8>) -> Result<PluginId, PluginError> {
        self.install_with(archive, self.source.clone(), self.verification.clone())
            .await
    }

    async fn install_with(
        &self,
        archive: Vec<u8>,
        source: Arc<dyn PluginSourceStorage>,
        verification: Arc<dyn PluginVerificationStorage>,
    ) -> Result<PluginId, PluginError> {
        InstallPluginFromProviderUseCase::new(
            self.registry.clone(),
            Arc::new(ZipPluginExtractor::default()),
            Arc::new(FsPluginStorage::new(
                self.location_info.clone(),
                self.verification.clone(),
                None,
            )),
            source,
            verification,
            self.settings.clone(),
            Arc::new(PluginProviderFactory::new(vec![Box::new(Download {
                archive,
            })])),
            self.sync.clone(),
        )
        .execute(
            PluginSourceType::GitHub,
            "owner/repo",
            ProviderRelease {
                download_url: "https://example.com/v2.zip".into(),
                tag_name: "v2".into(),
                version: "0.2.0".into(),
            },
        )
        .await
    }

    fn plugin_dir(&self) -> std::path::PathBuf {
        self.location_info.plugin_dir(&plugin_id())
    }

    /// What `plugins/` holds: the plugin's directory, and no copy of a replacement beside it.
    fn plugins_dir_entries(&self) -> Vec<std::ffi::OsString> {
        match std::fs::read_dir(self.location_info.plugins_dir()) {
            Ok(entries) => entries.map(|entry| entry.unwrap().file_name()).collect(),
            Err(_) => Vec::new(),
        }
    }
}

#[tokio::test]
async fn should_record_the_release_and_sync_when_a_plugin_is_installed() {
    let fixture = Fixture::empty();

    let plugin_id = fixture.install(plugin_zip(b"\0asm v2")).await.unwrap();

    assert_eq!(plugin_id.as_str(), PLUGIN_ID);
    assert_eq!(
        std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
        b"\0asm v2"
    );
    assert!(
        fixture
            .verification
            .get(&plugin_id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(matches!(
        fixture.source.get(&plugin_id).await.unwrap(),
        Some(PluginSource::Remote { current_tag, .. }) if current_tag == "v2"
    ));
    assert_eq!(fixture.sync.0.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn should_leave_nothing_behind_when_saving_the_source_fails_on_a_first_install() {
    let fixture = Fixture::empty();

    let result = fixture
        .install_with(
            plugin_zip(b"\0asm v2"),
            Arc::new(FailingSourceSave(fixture.source.clone())),
            fixture.verification.clone(),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(
        fixture.plugins_dir_entries(),
        Vec::<std::ffi::OsString>::new()
    );
    assert_eq!(fixture.sync.0.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn should_leave_nothing_behind_when_recording_trust_fails_on_a_first_install() {
    let fixture = Fixture::empty();

    let result = fixture
        .install_with(
            plugin_zip(b"\0asm v2"),
            fixture.source.clone(),
            Arc::new(FailingVerificationSave(fixture.verification.clone())),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(
        fixture.plugins_dir_entries(),
        Vec::<std::ffi::OsString>::new()
    );
}

#[tokio::test]
async fn should_write_nothing_when_the_archive_cannot_be_read() {
    let fixture = Fixture::empty();

    let result = fixture.install(b"not a zip".to_vec()).await;

    assert!(result.is_err());
    assert_eq!(
        fixture.plugins_dir_entries(),
        Vec::<std::ffi::OsString>::new()
    );
    assert_eq!(fixture.sync.0.load(Ordering::SeqCst), 0);
}

/// Version `v1` installed from GitHub, with settings the user changed.
async fn installed(fixture: &Fixture) {
    fixture.install(plugin_zip(b"\0asm v1")).await.unwrap();
    fixture
        .settings
        .upsert(
            &plugin_id(),
            &PluginSettings {
                allowed_hosts: vec!["extra.example.com".into()],
                ..PluginSettings::default()
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn should_keep_the_installed_version_when_replacing_it_fails_midway() {
    let fixture = Fixture::empty();
    installed(&fixture).await;
    let trusted = fixture.verification.get(&plugin_id()).await.unwrap();

    let result = fixture
        .install_with(
            plugin_zip(b"\0asm v2"),
            Arc::new(FailingSourceSave(fixture.source.clone())),
            fixture.verification.clone(),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(
        std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
        b"\0asm v1"
    );
    assert_eq!(
        fixture
            .verification
            .get(&plugin_id())
            .await
            .unwrap()
            .map(|record| record.wasm_sha256),
        trusted.map(|record| record.wasm_sha256)
    );
    assert_eq!(
        fixture
            .settings
            .get(&plugin_id())
            .await
            .unwrap()
            .unwrap()
            .allowed_hosts,
        ["extra.example.com"]
    );
    assert_eq!(fixture.plugins_dir_entries(), [PLUGIN_ID]);
}

#[tokio::test]
async fn should_replace_the_files_and_keep_the_settings_when_the_plugin_is_installed_again() {
    let fixture = Fixture::empty();
    installed(&fixture).await;
    std::fs::write(fixture.plugin_dir().join("leftover.txt"), b"old").unwrap();

    fixture.install(plugin_zip(b"\0asm v2")).await.unwrap();

    assert_eq!(
        std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
        b"\0asm v2"
    );
    assert!(
        !fixture.plugin_dir().join("leftover.txt").exists(),
        "a file of the old version must not stay beside the new one"
    );
    assert_eq!(
        fixture
            .settings
            .get(&plugin_id())
            .await
            .unwrap()
            .unwrap()
            .allowed_hosts,
        ["extra.example.com"]
    );
    assert_eq!(fixture.plugins_dir_entries(), [PLUGIN_ID]);
}
