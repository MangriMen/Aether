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

use crate::features::events::test_support::NoOpEmitter;
use crate::features::instance::PluginId;
use crate::features::{
    plugins::{
        InstallPluginFromProviderUseCase, PluginError, PluginProvider, PluginProviderError,
        PluginProviderFactory, PluginRegistry, PluginSettings, PluginSettingsStorage, PluginSource,
        PluginSourceStorage, PluginSourceType, PluginSyncService, PluginVerification,
        PluginVerificationStorage, PreviewPluginReleaseUseCase, ProviderPluginPreview,
        ProviderReleaseInfo, ProviderUpdateInfo, ReleaseArchiveCache,
        infra::{
            FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
            FsPluginVerificationStorage, ZipPluginExtractor,
        },
    },
    settings::LocationInfo,
};
use crate::shared::hash::infra::sha256_hex;

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
    downloads: Arc<AtomicUsize>,
}

impl Download {
    fn new(archive: Vec<u8>) -> Self {
        Self {
            archive,
            downloads: Arc::default(),
        }
    }
}

#[async_trait]
impl PluginProvider for Download {
    async fn fetch_preview(&self, _: &str) -> Result<ProviderPluginPreview, PluginProviderError> {
        Ok(ProviderPluginPreview {
            owner: "owner".into(),
            repo: "repo".into(),
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
        unimplemented!("the install does not look for updates")
    }

    async fn download_plugin(
        &self,
        _: &str,
        dest: &mut tokio::fs::File,
    ) -> Result<u64, PluginProviderError> {
        use tokio::io::AsyncWriteExt;

        self.downloads.fetch_add(1, Ordering::SeqCst);
        dest.write_all(&self.archive).await.unwrap();
        Ok(self.archive.len() as u64)
    }

    fn source_type(&self) -> PluginSourceType {
        PluginSourceType::GitHub
    }

    fn display_name(&self) -> String {
        "Stub source".to_owned()
    }

    fn parse_identifier(&self, raw: &str) -> Result<String, PluginProviderError> {
        Ok(raw.to_owned())
    }
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
        let sha256 = sha256_hex(&archive);
        self.install_download(Download::new(archive), &sha256, source, verification)
            .await
    }

    /// An install of `archive` that expects the previewed archive to hash to `expected_sha256`.
    async fn install_expecting(
        &self,
        archive: Vec<u8>,
        expected_sha256: &str,
    ) -> Result<PluginId, PluginError> {
        self.install_download(
            Download::new(archive),
            expected_sha256,
            self.source.clone(),
            self.verification.clone(),
        )
        .await
    }

    async fn install_download(
        &self,
        download: Download,
        expected_sha256: &str,
        source: Arc<dyn PluginSourceStorage>,
        verification: Arc<dyn PluginVerificationStorage>,
    ) -> Result<PluginId, PluginError> {
        let factory = Arc::new(PluginProviderFactory::new(vec![Box::new(download)]));
        self.use_case(
            source,
            verification,
            factory,
            Arc::new(ReleaseArchiveCache::new()),
        )
        .execute(
            PluginSourceType::GitHub,
            "owner/repo",
            "v2",
            expected_sha256,
        )
        .await
    }

    fn use_case(
        &self,
        source: Arc<dyn PluginSourceStorage>,
        verification: Arc<dyn PluginVerificationStorage>,
        factory: Arc<PluginProviderFactory>,
        cache: Arc<ReleaseArchiveCache>,
    ) -> InstallPluginFromProviderUseCase {
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
            factory,
            self.sync.clone(),
            cache,
        )
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

#[tokio::test]
async fn should_install_when_the_archive_hashes_to_what_was_previewed() {
    let fixture = Fixture::empty();
    let archive = plugin_zip(b"\0asm v2");
    // The case of the hex does not matter.
    let expected = sha256_hex(&archive).to_uppercase();

    let result = fixture.install_expecting(archive, &expected).await;

    assert!(result.is_ok(), "{result:?}");
}

#[tokio::test]
async fn should_refuse_and_write_nothing_when_the_archive_hashes_to_something_else() {
    let fixture = Fixture::empty();

    let result = fixture
        .install_expecting(plugin_zip(b"\0asm v2"), &sha256_hex(b"another archive"))
        .await;

    assert!(matches!(
        result,
        Err(PluginError::Provider(
            PluginProviderError::ArchiveChangedSincePreview
        ))
    ));
    assert_eq!(
        fixture.plugins_dir_entries(),
        Vec::<std::ffi::OsString>::new()
    );
    assert!(fixture.source.get(&plugin_id()).await.unwrap().is_none());
    assert!(
        fixture
            .verification
            .get(&plugin_id())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(fixture.sync.0.load(Ordering::SeqCst), 0);
}

/// Previews `v2` and then installs it with `expected_sha256`, the way the app does, through one
/// cache. Gives the result of the install and how many times the provider was asked to download.
async fn preview_then_install(
    fixture: &Fixture,
    archive: Vec<u8>,
    expected_sha256: Option<&str>,
) -> (Result<PluginId, PluginError>, usize) {
    let sha256 = sha256_hex(&archive);
    let download = Download::new(archive);
    let downloads = download.downloads.clone();
    let factory = Arc::new(PluginProviderFactory::new(vec![Box::new(download)]));
    let cache = Arc::new(ReleaseArchiveCache::new());

    PreviewPluginReleaseUseCase::new(
        Arc::new(ZipPluginExtractor::default()),
        factory.clone(),
        cache.clone(),
    )
    .execute(PluginSourceType::GitHub, "owner/repo", Some("v2".into()))
    .await
    .unwrap();
    let result = fixture
        .use_case(
            fixture.source.clone(),
            fixture.verification.clone(),
            factory,
            cache,
        )
        .execute(
            PluginSourceType::GitHub,
            "owner/repo",
            "v2",
            expected_sha256.unwrap_or(&sha256),
        )
        .await;
    (result, downloads.load(Ordering::SeqCst))
}

#[tokio::test]
async fn should_download_once_when_a_release_is_installed_after_its_preview() {
    let fixture = Fixture::empty();

    let (result, downloads) = preview_then_install(&fixture, plugin_zip(b"\0asm v2"), None).await;

    assert!(result.is_ok(), "{result:?}");
    assert_eq!(downloads, 1);
}

#[tokio::test]
async fn should_refuse_an_archive_from_the_cache_that_hashes_to_something_else() {
    let fixture = Fixture::empty();

    let (result, downloads) = preview_then_install(
        &fixture,
        plugin_zip(b"\0asm v2"),
        Some(&sha256_hex(b"another archive")),
    )
    .await;

    assert!(matches!(
        result,
        Err(PluginError::Provider(
            PluginProviderError::ArchiveChangedSincePreview
        ))
    ));
    assert_eq!(downloads, 1);
    assert_eq!(
        fixture.plugins_dir_entries(),
        Vec::<std::ffi::OsString>::new()
    );
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

#[test]
fn should_list_each_provider_with_the_name_it_gives_itself() {
    let factory = PluginProviderFactory::new(vec![Box::new(Download::new(Vec::new()))]);

    assert_eq!(
        factory.list_providers(),
        [crate::features::plugins::PluginProviderInfo {
            source_type: PluginSourceType::GitHub,
            name: "Stub source".to_owned(),
        }]
    );
}
