use std::{
    io::{Cursor, Write},
    sync::Arc,
};

use async_trait::async_trait;
use tempfile::TempDir;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::features::{
    events::{Event, EventEmitter, EventError},
    plugins::{
        ImportPluginsUseCase, PluginError, PluginRegistry, PluginSettingsStorage, PluginSource,
        PluginSourceStorage, PluginSourceType, PluginSyncService, PluginVerification,
        PluginVerificationStorage,
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
        "version": "0.1.0",
        "description": null,
        "authors": [],
        "license": null
    },
    "runtime": { "allowedHosts": [], "allowedPaths": [] },
    "load": { "type": "extism", "file": "plugin.wasm", "memoryLimit": null },
    "api": { "version": "*", "features": [] }
}"#;

struct NoOpEmitter;

#[async_trait]
impl EventEmitter<Event> for NoOpEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }
    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

fn registry() -> Arc<PluginRegistry> {
    Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter)))
}

fn settings(location_info: &Arc<LocationInfo>) -> Arc<FsPluginSettingsStorage> {
    Arc::new(FsPluginSettingsStorage::new(location_info.clone()))
}

async fn saved_hosts(location_info: &Arc<LocationInfo>) -> Vec<String> {
    settings(location_info)
        .get(PLUGIN_ID)
        .await
        .unwrap()
        .unwrap()
        .allowed_hosts
}
struct NoSync;

#[async_trait]
impl PluginSyncService for NoSync {
    async fn execute(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

fn plugin_zip(wasm: &[u8]) -> Vec<u8> {
    plugin_zip_with(wasm, &[])
}

fn plugin_zip_with(wasm: &[u8], extra: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in [
        ("manifest.json", MANIFEST.as_bytes()),
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

/// A plugin installed from a provider, then imported from a local build of it: the provider's
/// records go with the old bytes — the recorded sha256 would refuse the new wasm, the source
/// would let an update replace it — and the user's settings stay.
#[tokio::test]
async fn should_drop_the_provider_records_when_a_plugin_is_imported_over_another() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().join("config"),
    ));
    let verification = Arc::new(FsPluginVerificationStorage::new(location_info.clone()));
    let source = Arc::new(FsPluginSourceStorage::new(location_info.clone()));
    let storage = Arc::new(FsPluginStorage::new(
        location_info.clone(),
        verification.clone(),
        None,
    ));

    let plugin_dir = location_info.plugin_dir(PLUGIN_ID);
    tokio::fs::create_dir_all(&plugin_dir).await.unwrap();
    tokio::fs::write(plugin_dir.join("plugin.wasm"), b"\0asm released")
        .await
        .unwrap();
    tokio::fs::write(
        location_info.plugin_settings(PLUGIN_ID),
        "allowedHosts = [\"kept.example.com\"]\nallowedPaths = []\n",
    )
    .await
    .unwrap();
    verification
        .save(
            PLUGIN_ID,
            &PluginVerification::new("sha256-of-release", PluginSourceType::GitHub),
        )
        .await
        .unwrap();
    source
        .save(
            PLUGIN_ID,
            &PluginSource::Remote {
                source_type: PluginSourceType::GitHub,
                identifier: "owner/repo".into(),
                current_tag: "v1".into(),
                current_version: "1.0.0".into(),
            },
        )
        .await
        .unwrap();

    let archive = temp.path().join("local.zip");
    tokio::fs::write(&archive, plugin_zip(b"\0asm local build"))
        .await
        .unwrap();

    ImportPluginsUseCase::new(
        registry(),
        settings(&location_info),
        Arc::new(ZipPluginExtractor::default()),
        storage,
        source.clone(),
        verification.clone(),
        Arc::new(NoSync),
    )
    .execute(vec![archive])
    .await
    .unwrap();

    assert_eq!(verification.get(PLUGIN_ID).await.unwrap(), None);
    assert_eq!(source.get(PLUGIN_ID).await.unwrap(), None);
    assert_eq!(
        tokio::fs::read(plugin_dir.join("plugin.wasm"))
            .await
            .unwrap(),
        b"\0asm local build"
    );
    assert_eq!(saved_hosts(&location_info).await, vec!["kept.example.com"]);
}

/// A first import has no records to drop.
#[tokio::test]
async fn should_import_a_plugin_when_it_has_no_records() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().join("config"),
    ));
    let verification = Arc::new(FsPluginVerificationStorage::new(location_info.clone()));
    let source = Arc::new(FsPluginSourceStorage::new(location_info.clone()));

    let archive = temp.path().join("local.zip");
    tokio::fs::write(&archive, plugin_zip(b"\0asm"))
        .await
        .unwrap();

    ImportPluginsUseCase::new(
        registry(),
        settings(&location_info),
        Arc::new(ZipPluginExtractor::default()),
        Arc::new(FsPluginStorage::new(
            location_info.clone(),
            verification.clone(),
            None,
        )),
        source,
        verification,
        Arc::new(NoSync),
    )
    .execute(vec![archive])
    .await
    .unwrap();

    assert!(
        location_info
            .plugin_dir(PLUGIN_ID)
            .join("plugin.wasm")
            .exists()
    );
}

/// Files the launcher keeps about a plugin, as an archive could bring them: settings granting a
/// host, a trust record and a source of its own. One in another case, as Windows would read it.
const PLANTED_CORE_FILES: [(&str, &[u8]); 3] = [
    (
        "settings.toml",
        b"allowed_hosts = [\"evil.example.com\"]\n",
    ),
    (
        "Verification.JSON",
        br#"{"sha256":"0000","sourceType":"github"}"#,
    ),
    (
        "source.json",
        br#"{"type":"remote","sourceType":"github","identifier":"evil/repo","currentTag":"v9","currentVersion":"9.0.0"}"#,
    ),
];

fn has_file_like(dir: &std::path::Path, name: &str) -> bool {
    std::fs::read_dir(dir).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
    })
}

#[tokio::test]
async fn should_leave_out_core_files_when_an_imported_archive_brings_them() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().join("config"),
    ));
    let verification = Arc::new(FsPluginVerificationStorage::new(location_info.clone()));
    let source = Arc::new(FsPluginSourceStorage::new(location_info.clone()));

    let archive = temp.path().join("local.zip");
    tokio::fs::write(&archive, plugin_zip_with(b"\0asm", &PLANTED_CORE_FILES))
        .await
        .unwrap();

    ImportPluginsUseCase::new(
        registry(),
        settings(&location_info),
        Arc::new(ZipPluginExtractor::default()),
        Arc::new(FsPluginStorage::new(
            location_info.clone(),
            verification.clone(),
            None,
        )),
        source.clone(),
        verification.clone(),
        Arc::new(NoSync),
    )
    .execute(vec![archive])
    .await
    .unwrap();

    let plugin_dir = location_info.plugin_dir(PLUGIN_ID);
    assert!(plugin_dir.join("plugin.wasm").exists());
    for name in ["settings.toml", "verification.json", "source.json"] {
        assert!(!has_file_like(&plugin_dir, name), "{name} was copied");
    }
    assert_eq!(verification.get(PLUGIN_ID).await.unwrap(), None);
    assert_eq!(source.get(PLUGIN_ID).await.unwrap(), None);
}

#[tokio::test]
async fn should_keep_the_user_settings_when_an_imported_archive_brings_its_own() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().join("config"),
    ));
    let verification = Arc::new(FsPluginVerificationStorage::new(location_info.clone()));
    let source = Arc::new(FsPluginSourceStorage::new(location_info.clone()));

    tokio::fs::create_dir_all(location_info.plugin_dir(PLUGIN_ID))
        .await
        .unwrap();
    tokio::fs::write(
        location_info.plugin_settings(PLUGIN_ID),
        "allowedHosts = [\"kept.example.com\"]\nallowedPaths = []\n",
    )
    .await
    .unwrap();

    let archive = temp.path().join("local.zip");
    tokio::fs::write(&archive, plugin_zip_with(b"\0asm", &PLANTED_CORE_FILES))
        .await
        .unwrap();

    ImportPluginsUseCase::new(
        registry(),
        settings(&location_info),
        Arc::new(ZipPluginExtractor::default()),
        Arc::new(FsPluginStorage::new(
            location_info.clone(),
            verification.clone(),
            None,
        )),
        source,
        verification,
        Arc::new(NoSync),
    )
    .execute(vec![archive])
    .await
    .unwrap();

    assert_eq!(saved_hosts(&location_info).await, vec!["kept.example.com"]);
}

struct CountingSync(std::sync::atomic::AtomicUsize);

#[async_trait]
impl PluginSyncService for CountingSync {
    async fn execute(&self) -> Result<(), PluginError> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn should_sync_and_report_the_error_when_a_later_archive_fails() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().join("config"),
    ));
    let verification = Arc::new(FsPluginVerificationStorage::new(location_info.clone()));
    let source = Arc::new(FsPluginSourceStorage::new(location_info.clone()));

    let good = temp.path().join("good.zip");
    tokio::fs::write(&good, plugin_zip(b"\0asm")).await.unwrap();
    let missing = temp.path().join("missing.zip");

    let sync = Arc::new(CountingSync(std::sync::atomic::AtomicUsize::new(0)));
    let result = ImportPluginsUseCase::new(
        registry(),
        settings(&location_info),
        Arc::new(ZipPluginExtractor::default()),
        Arc::new(FsPluginStorage::new(
            location_info.clone(),
            verification.clone(),
            None,
        )),
        source,
        verification,
        sync.clone(),
    )
    .execute(vec![good, missing])
    .await;

    assert!(result.is_err());
    assert_eq!(sync.0.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(
        location_info
            .plugin_dir(PLUGIN_ID)
            .join("plugin.wasm")
            .exists()
    );
}

/// An imported plugin replaces the installed one as a whole: a file only the old build had is
/// gone, and the user's settings carry over.
#[tokio::test]
async fn should_replace_the_installed_plugin_when_one_of_its_id_is_imported() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().join("config"),
    ));
    let verification = Arc::new(FsPluginVerificationStorage::new(location_info.clone()));
    let source = Arc::new(FsPluginSourceStorage::new(location_info.clone()));
    let storage = Arc::new(FsPluginStorage::new(
        location_info.clone(),
        verification.clone(),
        None,
    ));

    let plugin_dir = location_info.plugin_dir(PLUGIN_ID);
    tokio::fs::create_dir_all(&plugin_dir).await.unwrap();
    tokio::fs::write(plugin_dir.join("stale.bin"), b"old")
        .await
        .unwrap();
    tokio::fs::write(
        location_info.plugin_settings(PLUGIN_ID),
        "allowedHosts = [\"kept.example.com\"]\nallowedPaths = []\n",
    )
    .await
    .unwrap();

    let archive = temp.path().join("local.zip");
    tokio::fs::write(&archive, plugin_zip(b"\0asm new"))
        .await
        .unwrap();

    ImportPluginsUseCase::new(
        registry(),
        settings(&location_info),
        Arc::new(ZipPluginExtractor::default()),
        storage,
        source,
        verification,
        Arc::new(NoSync),
    )
    .execute(vec![archive])
    .await
    .unwrap();

    assert!(!plugin_dir.join("stale.bin").exists());
    assert_eq!(
        tokio::fs::read(plugin_dir.join("plugin.wasm"))
            .await
            .unwrap(),
        b"\0asm new"
    );
    assert_eq!(saved_hosts(&location_info).await, vec!["kept.example.com"]);
    assert!(
        std::fs::read_dir(location_info.plugins_dir())
            .unwrap()
            .count()
            == 1,
        "a copy of the old build was left in plugins/"
    );
}
