use std::{
    collections::HashMap,
    io::{Cursor, Write},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use tempfile::TempDir;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::features::{
    events::{Event, EventEmitter, EventError},
    plugins::{
        AllowedPath, ExtractedPlugin, GrantPluginFolderAccessUseCase, Plugin, PluginError,
        PluginProvider, PluginProviderError, PluginProviderFactory, PluginRegistry, PluginSettings,
        PluginSettingsStorage, PluginSource, PluginSourceStorage, PluginSourceType, PluginState,
        PluginStorage, PluginVerification, PluginVerificationStorage, ProviderPluginPreview,
        ProviderUpdateInfo, ReloadPluginUseCasePort, UpdatePluginUseCase,
        domain::ProviderReleaseInfo,
        infra::{
            FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
            FsPluginVerificationStorage, ZipPluginExtractor,
        },
    },
    settings::LocationInfo,
};

const PLUGIN_ID: &str = "test-plugin";

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
    plugin_zip_with(wasm, &[])
}

fn plugin_zip_with(wasm: &[u8], extra: &[(&str, &[u8])]) -> Vec<u8> {
    archive(MANIFEST, wasm, extra)
}

fn plugin_zip_of(id: &str, wasm: &[u8]) -> Vec<u8> {
    archive(&MANIFEST.replace(PLUGIN_ID, id), wasm, &[])
}

fn archive(manifest: &str, wasm: &[u8], extra: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in [
        ("manifest.json", manifest.as_bytes()),
        ("plugin.wasm", wasm),
    ]
    .into_iter()
    .chain(extra.iter().copied())
    {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// A GitHub-like source with one release, `v2`, whose archive is `archive`.
struct OneRelease {
    archive: Vec<u8>,
}

#[async_trait]
impl PluginProvider for OneRelease {
    async fn fetch_preview(&self, _: &str) -> Result<ProviderPluginPreview, PluginProviderError> {
        Ok(ProviderPluginPreview {
            owner: "owner".into(),
            repo: "repo".into(),
            manifest: None,
            capabilities: None,
            releases: vec![ProviderReleaseInfo {
                tag_name: "v2".into(),
                version: "0.2.0".into(),
                is_prerelease: false,
                published_at: "2026-01-01T00:00:00Z".into(),
                html_url: "https://example.com/v2".into(),
                download_url: "https://example.com/v2.zip".into(),
            }],
        })
    }

    async fn fetch_latest_version(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<ProviderUpdateInfo, PluginProviderError> {
        unimplemented!("the tests name the release")
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

/// The plugin is not running in these tests, so nothing restarts it.
struct NoRestart;

#[async_trait]
impl ReloadPluginUseCasePort for NoRestart {
    async fn execute(&self, _plugin_id: String) -> Result<(), PluginError> {
        unreachable!("a plugin that is not running is not restarted")
    }
}

/// Hands out what it holds, then takes its time before returning it: two changes that were not
/// made one after the other both read the settings before either saves.
struct SlowPluginSettings(Arc<FsPluginSettingsStorage>);

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

struct Fixture {
    _temp: TempDir,
    location_info: Arc<LocationInfo>,
    registry: Arc<PluginRegistry>,
    settings: Arc<FsPluginSettingsStorage>,
    source: Arc<FsPluginSourceStorage>,
    verification: Arc<FsPluginVerificationStorage>,
}

impl Fixture {
    /// Version `v1` of the plugin installed from GitHub, with settings the user changed.
    async fn installed() -> Self {
        let temp = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().join("config"),
        ));
        let registry = Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter)));
        let manifest: aether_core_plugin_api::v0::PluginManifestDto =
            serde_json::from_str(MANIFEST).unwrap();
        registry.insert(
            PLUGIN_ID.to_owned(),
            Plugin {
                manifest: manifest.try_into().unwrap(),
                capabilities: None,
                hash: "hash".into(),
                verified: true,
                state: PluginState::NotLoaded,
            },
        );
        let fixture = Self {
            registry,
            settings: Arc::new(FsPluginSettingsStorage::new(location_info.clone())),
            source: Arc::new(FsPluginSourceStorage::new(location_info.clone())),
            verification: Arc::new(FsPluginVerificationStorage::new(location_info.clone())),
            location_info,
            _temp: temp,
        };

        let plugin_dir = fixture.location_info.plugin_dir(PLUGIN_ID);
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();
        tokio::fs::write(plugin_dir.join("plugin.wasm"), b"\0asm v1")
            .await
            .unwrap();
        fixture
            .source
            .save(
                PLUGIN_ID,
                &PluginSource::Remote {
                    source_type: PluginSourceType::GitHub,
                    identifier: "owner/repo".into(),
                    current_tag: "v1".into(),
                    current_version: "0.1.0".into(),
                },
            )
            .await
            .unwrap();
        fixture
            .verification
            .save(
                PLUGIN_ID,
                &PluginVerification::new("sha256 of v1", PluginSourceType::GitHub),
            )
            .await
            .unwrap();
        fixture
            .settings
            .upsert(
                PLUGIN_ID,
                &PluginSettings {
                    allowed_hosts: vec!["extra.example.com".into()],
                    force_enabled_at_api_version: Some("0.1.0".into()),
                    ..PluginSettings::default()
                },
            )
            .await
            .unwrap();

        fixture
    }

    async fn update_to_v2(&self, archive: Vec<u8>) -> Result<(), PluginError> {
        self.update_to_v2_with(archive, Seams::default()).await
    }

    async fn update_to_v2_with(&self, archive: Vec<u8>, seams: Seams) -> Result<(), PluginError> {
        let storage = Arc::new(FsPluginStorage::new(
            self.location_info.clone(),
            self.verification.clone(),
            None,
        ));
        UpdatePluginUseCase::new(
            self.registry.clone(),
            Arc::new(ZipPluginExtractor::default()),
            if seams.failing_replace {
                Arc::new(FailingReplace(storage))
            } else {
                storage
            },
            seams.source.unwrap_or_else(|| self.source.clone()),
            seams
                .verification
                .unwrap_or_else(|| self.verification.clone()),
            seams.settings.unwrap_or_else(|| self.settings.clone()),
            Arc::new(PluginProviderFactory::new(vec![Box::new(OneRelease {
                archive,
            })])),
        )
        .execute(PLUGIN_ID, Some("v2"))
        .await
    }

    async fn wasm(&self) -> Vec<u8> {
        tokio::fs::read(self.location_info.plugin_dir(PLUGIN_ID).join("plugin.wasm"))
            .await
            .unwrap()
    }

    /// Version `v1` as [`Fixture::installed`] left it, and nothing of the update beside it.
    async fn assert_v1_untouched(&self) {
        assert_eq!(self.wasm().await, b"\0asm v1");
        let settings = self.settings.get(PLUGIN_ID).await.unwrap().unwrap();
        assert_eq!(settings.allowed_hosts, ["extra.example.com"]);
        assert_eq!(
            settings.force_enabled_at_api_version.as_deref(),
            Some("0.1.0")
        );
        assert_eq!(
            self.verification
                .get(PLUGIN_ID)
                .await
                .unwrap()
                .map(|record| record.wasm_sha256),
            Some("sha256 of v1".to_owned())
        );
        assert!(matches!(
            self.source.get(PLUGIN_ID).await.unwrap(),
            Some(PluginSource::Remote { current_tag, .. }) if current_tag == "v1"
        ));
        let entries: Vec<_> = std::fs::read_dir(self.location_info.plugins_dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(
            entries,
            [PLUGIN_ID],
            "nothing of the update is left beside it"
        );
    }
}

/// Parts of the update to replace with ones that fail.
#[derive(Default)]
struct Seams {
    failing_replace: bool,
    settings: Option<Arc<dyn PluginSettingsStorage>>,
    verification: Option<Arc<dyn PluginVerificationStorage>>,
    source: Option<Arc<dyn PluginSourceStorage>>,
}

fn disk_full() -> PluginError {
    PluginError::Storage(std::io::Error::other("disk full").into())
}

struct FailingReplace(Arc<FsPluginStorage>);

#[async_trait]
impl PluginStorage for FailingReplace {
    async fn add(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError> {
        self.0.add(extracted_plugin).await
    }
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError> {
        self.0.list().await
    }
    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError> {
        self.0.get(plugin_id).await
    }
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.0.remove(plugin_id).await
    }
    async fn replace(&self, _extracted_plugin: ExtractedPlugin) -> Result<(), PluginError> {
        Err(disk_full())
    }
    async fn restore_replaced(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.0.restore_replaced(plugin_id).await
    }
    async fn discard_replaced(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.0.discard_replaced(plugin_id).await
    }
    async fn wasm_sha256(&self, plugin_id: &str) -> Result<String, PluginError> {
        self.0.wasm_sha256(plugin_id).await
    }
}

struct FailingSettingsSave(Arc<FsPluginSettingsStorage>);

#[async_trait]
impl PluginSettingsStorage for FailingSettingsSave {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        self.0.get(plugin_id).await
    }
    async fn upsert(&self, _: &str, _: &PluginSettings) -> Result<(), PluginError> {
        Err(disk_full())
    }
}

struct FailingVerificationSave(Arc<FsPluginVerificationStorage>);

#[async_trait]
impl PluginVerificationStorage for FailingVerificationSave {
    async fn save(&self, _: &str, _: &PluginVerification) -> Result<(), PluginError> {
        Err(disk_full())
    }
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginVerification>, PluginError> {
        self.0.get(plugin_id).await
    }
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.0.remove(plugin_id).await
    }
}

struct FailingSourceSave(Arc<FsPluginSourceStorage>);

#[async_trait]
impl PluginSourceStorage for FailingSourceSave {
    async fn save(&self, _: &str, _: &PluginSource) -> Result<(), PluginError> {
        Err(disk_full())
    }
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSource>, PluginError> {
        self.0.get(plugin_id).await
    }
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.0.remove(plugin_id).await
    }
}

#[tokio::test]
async fn should_keep_the_installed_version_when_the_new_files_cannot_be_put_in_place() {
    let fixture = Fixture::installed().await;

    let result = fixture
        .update_to_v2_with(
            plugin_zip(b"\0asm v2"),
            Seams {
                failing_replace: true,
                ..Seams::default()
            },
        )
        .await;

    assert!(result.is_err());
    fixture.assert_v1_untouched().await;
}

#[tokio::test]
async fn should_keep_the_installed_version_when_the_settings_cannot_be_saved() {
    let fixture = Fixture::installed().await;

    let result = fixture
        .update_to_v2_with(
            plugin_zip(b"\0asm v2"),
            Seams {
                settings: Some(Arc::new(FailingSettingsSave(fixture.settings.clone()))),
                ..Seams::default()
            },
        )
        .await;

    assert!(result.is_err());
    fixture.assert_v1_untouched().await;
}

#[tokio::test]
async fn should_keep_the_installed_version_when_the_verification_cannot_be_recorded() {
    let fixture = Fixture::installed().await;

    let result = fixture
        .update_to_v2_with(
            plugin_zip(b"\0asm v2"),
            Seams {
                verification: Some(Arc::new(FailingVerificationSave(
                    fixture.verification.clone(),
                ))),
                ..Seams::default()
            },
        )
        .await;

    assert!(result.is_err());
    fixture.assert_v1_untouched().await;
}

#[tokio::test]
async fn should_keep_the_installed_version_when_the_source_cannot_be_saved() {
    let fixture = Fixture::installed().await;

    let result = fixture
        .update_to_v2_with(
            plugin_zip(b"\0asm v2"),
            Seams {
                source: Some(Arc::new(FailingSourceSave(fixture.source.clone()))),
                ..Seams::default()
            },
        )
        .await;

    assert!(result.is_err());
    fixture.assert_v1_untouched().await;
}

#[tokio::test]
async fn should_leave_nothing_beside_the_plugin_when_the_update_succeeds() {
    let fixture = Fixture::installed().await;

    fixture.update_to_v2(plugin_zip(b"\0asm v2")).await.unwrap();

    let entries: Vec<_> = std::fs::read_dir(fixture.location_info.plugins_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(entries, [PLUGIN_ID]);
}

/// The settings live in the plugin's directory, which the update replaces: they carry over, but
/// a force-enable was confirmed for the old build and is asked again.
#[tokio::test]
async fn should_keep_the_settings_of_a_plugin_when_it_is_updated() {
    let fixture = Fixture::installed().await;

    fixture.update_to_v2(plugin_zip(b"\0asm v2")).await.unwrap();

    assert_eq!(fixture.wasm().await, b"\0asm v2");
    let settings = fixture.settings.get(PLUGIN_ID).await.unwrap().unwrap();
    assert_eq!(settings.allowed_hosts, ["extra.example.com"]);
    assert_eq!(settings.force_enabled_at_api_version, None);
    assert!(fixture.verification.get(PLUGIN_ID).await.unwrap().is_some());
    assert!(matches!(
        fixture.source.get(PLUGIN_ID).await.unwrap(),
        Some(PluginSource::Remote { current_tag, .. }) if current_tag == "v2"
    ));
}

/// A release whose archive cannot be read leaves the installed version as it was.
#[tokio::test]
async fn should_keep_the_installed_version_when_the_new_archive_is_broken() {
    let fixture = Fixture::installed().await;

    fixture
        .update_to_v2(b"not a zip".to_vec())
        .await
        .unwrap_err();

    assert_eq!(fixture.wasm().await, b"\0asm v1");
    assert!(fixture.settings.get(PLUGIN_ID).await.unwrap().is_some());
}

/// With no settings of the user's to carry over, nothing would replace a `settings.toml` of the
/// release's own: the plugin would grant itself hosts. Its `source.json` is left out too.
#[tokio::test]
async fn should_leave_out_core_files_when_the_release_archive_brings_them() {
    let fixture = Fixture::installed().await;
    tokio::fs::remove_file(fixture.location_info.plugin_settings(PLUGIN_ID))
        .await
        .unwrap();

    fixture
        .update_to_v2(plugin_zip_with(
            b"\0asm v2",
            &[
                (
                    "settings.toml",
                    b"allowed_hosts = [\"evil.example.com\"]\n",
                ),
                (
                    "source.json",
                    br#"{"type":"remote","sourceType":"github","identifier":"evil/repo","currentTag":"v9","currentVersion":"9.0.0"}"#,
                ),
            ],
        ))
        .await
        .unwrap();

    assert_eq!(fixture.wasm().await, b"\0asm v2");
    assert!(fixture.settings.get(PLUGIN_ID).await.unwrap().is_none());
    assert!(matches!(
        fixture.source.get(PLUGIN_ID).await.unwrap(),
        Some(PluginSource::Remote { identifier, current_tag, .. })
            if identifier == "owner/repo" && current_tag == "v2"
    ));
}

/// An archive of another plugin would be installed under the id of the one it replaces, or over
/// a plugin it has nothing to do with: the installed version stays as it was.
#[tokio::test]
async fn should_refuse_the_update_when_the_archive_has_another_plugin_id() {
    let fixture = Fixture::installed().await;

    let result = fixture
        .update_to_v2(plugin_zip_of("other-plugin", b"\0asm v2"))
        .await;

    assert!(matches!(
        result,
        Err(PluginError::IdMismatch { plugin_id, archive_id })
            if plugin_id == PLUGIN_ID && archive_id == "other-plugin"
    ));
    assert_eq!(fixture.wasm().await, b"\0asm v1");
    let settings = fixture.settings.get(PLUGIN_ID).await.unwrap().unwrap();
    assert_eq!(settings.allowed_hosts, ["extra.example.com"]);
    assert_eq!(
        settings.force_enabled_at_api_version.as_deref(),
        Some("0.1.0")
    );
    assert!(!fixture.location_info.plugin_dir("other-plugin").exists());
}

/// The update reads the settings, replaces the plugin's directory and saves them back: a folder
/// granted in between would be saved over with the settings read before it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_keep_a_folder_granted_when_it_is_granted_during_an_update() {
    let fixture = Fixture::installed().await;
    let slow: Arc<dyn PluginSettingsStorage> =
        Arc::new(SlowPluginSettings(fixture.settings.clone()));
    let pack = TempDir::new().unwrap();
    let pack_file = pack.path().join("pack.toml");
    std::fs::write(&pack_file, b"").unwrap();
    let grant = GrantPluginFolderAccessUseCase::new(
        fixture.registry.clone(),
        slow.clone(),
        Arc::new(NoRestart),
        fixture.location_info.clone(),
    );

    let (updated, granted) = tokio::join!(
        fixture.update_to_v2_with(
            plugin_zip(b"\0asm v2"),
            Seams {
                settings: Some(slow),
                ..Seams::default()
            }
        ),
        grant.execute(PLUGIN_ID.into(), pack_file, false),
    );

    updated.unwrap();
    granted.unwrap();
    let settings = fixture.settings.get(PLUGIN_ID).await.unwrap().unwrap();
    assert_eq!(
        settings.allowed_paths,
        [AllowedPath::read_only_folder(pack.path())]
    );
    assert_eq!(settings.allowed_hosts, ["extra.example.com"]);
}
