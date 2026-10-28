//! The files of a game version as one shared download, with the real progress service and file
//! cache, and a request client serving fixed bodies.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::{mpsc, watch};

use crate::{
    features::{
        events::{
            Event, ProgressBarId, ProgressBarStorage, ProgressEventType, ProgressService,
            ProgressServiceImpl, ProgressStatus,
            infra::InMemoryProgressBarStorage,
            test_support::{ChannelEmitter, mirrors, pack, sent},
        },
        minecraft::{MinecraftDomainError, MinecraftDownloader, vanilla},
        settings::LocationInfo,
    },
    shared::{
        cache::infra::FileCache,
        hash::infra::{hash_hex_async, sha1_async},
        request_client::{BytesProgress, FileDownload, Request, RequestClient, RequestError},
    },
};

use super::{
    AssetsService, ClientService, LibrariesService, LoggingService, MinecraftDownloadResolver,
    MinecraftDownloadService,
};

const LIBRARY: &str = "com.example:lib:1.0";
const LIBRARY_URL: &str = "https://libraries.example/lib-1.0.jar";
const ASSET_HASH: &str = "ab0123456789";

/// Serves fixed bodies by URL and records every fetch; a gated URL answers only once the gate
/// is opened.
struct FakeClient {
    bodies: Mutex<HashMap<String, Bytes>>,
    fetched: Mutex<Vec<String>>,
    gated: Mutex<HashSet<String>>,
    gate: watch::Sender<bool>,
}

impl Default for FakeClient {
    fn default() -> Self {
        Self {
            bodies: Mutex::default(),
            fetched: Mutex::default(),
            gated: Mutex::default(),
            gate: watch::channel(false).0,
        }
    }
}

impl FakeClient {
    fn serve(&self, url: &str, body: impl Into<Bytes>) {
        self.bodies
            .lock()
            .unwrap()
            .insert(url.to_owned(), body.into());
    }

    fn gate(&self, url: &str) {
        self.gated.lock().unwrap().insert(url.to_owned());
    }

    fn open(&self) {
        self.gate.send_replace(true);
    }

    fn fetches_of(&self, url: &str) -> usize {
        self.fetched
            .lock()
            .unwrap()
            .iter()
            .filter(|fetched| *fetched == url)
            .count()
    }
}

#[async_trait]
impl RequestClient for FakeClient {
    async fn fetch_bytes(&self, request: Request) -> Result<Bytes, RequestError> {
        self.fetch_bytes_with_progress(request, None).await
    }

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        _loading_bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<Bytes, RequestError> {
        self.fetched.lock().unwrap().push(request.url.clone());
        if self.gated.lock().unwrap().contains(&request.url) {
            let mut gate = self.gate.subscribe();
            gate.wait_for(|open| *open).await.unwrap();
        }
        let body = self
            .bodies
            .lock()
            .unwrap()
            .get(&request.url)
            .cloned()
            .ok_or(RequestError::Status {
                status: 404,
                url: request.url.clone(),
            })?;
        // As the real client does: a body that is not the expected one is refused.
        if let Some(expected) = &request.hash {
            let actual = hash_hex_async(expected.algorithm(), body.clone())
                .await
                .unwrap();
            if !expected.matches(&actual) {
                return Err(RequestError::Hash {
                    actual,
                    expected: expected.hex().to_owned(),
                });
            }
        }
        Ok(body)
    }

    async fn download_to_file(
        &self,
        _: Request,
        _: &Path,
        _: u64,
        _: Option<BytesProgress<'_>>,
    ) -> Result<u64, RequestError> {
        unimplemented!("the game's files are fetched whole")
    }

    async fn download_to_file_conditional(
        &self,
        _: Request,
        _: &Path,
        _: u64,
    ) -> Result<FileDownload, RequestError> {
        unimplemented!("the game's files are fetched whole")
    }
}

type Cache = FileCache<MinecraftDownloadResolver>;
type Service = MinecraftDownloadService<
    FakeClient,
    ProgressServiceImpl<InMemoryProgressBarStorage>,
    Cache,
    Cache,
>;

struct Fixture {
    service: Arc<Service>,
    client: Arc<FakeClient>,
    progress: Arc<ProgressServiceImpl<InMemoryProgressBarStorage>>,
    storage: Arc<InMemoryProgressBarStorage>,
    events: mpsc::UnboundedReceiver<Event>,
    location_info: Arc<LocationInfo>,
    _temp: tempfile::TempDir,
}

fn fixture() -> Fixture {
    let temp = tempfile::TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let (tx, events) = mpsc::unbounded_channel();
    let storage = Arc::new(InMemoryProgressBarStorage::default());
    let progress = Arc::new(ProgressServiceImpl::new(
        Arc::new(ChannelEmitter(tx)),
        storage.clone(),
    ));
    let cache = Arc::new(FileCache::new(MinecraftDownloadResolver::new(
        location_info.clone(),
    )));
    let client = Arc::new(FakeClient::default());
    let service = Arc::new(MinecraftDownloadService::new(
        ClientService::new(client.clone(), cache.clone()),
        AssetsService::new(
            client.clone(),
            client.clone(),
            location_info.clone(),
            cache.clone(),
        ),
        LibrariesService::new(client.clone(), location_info.clone()),
        LoggingService::new(client.clone(), location_info.clone()),
        client.clone(),
        progress.clone(),
        cache,
    ));
    Fixture {
        service,
        client,
        progress,
        storage,
        events,
        location_info,
        _temp: temp,
    }
}

fn client_url(version: &str) -> String {
    format!("https://client.example/{version}.jar")
}

fn assets_index_url(version: &str) -> String {
    format!("https://assets.example/{version}.json")
}

fn asset_url(hash: &str) -> String {
    format!(
        "https://resources.download.minecraft.net/{}/{hash}",
        &hash[..2]
    )
}

/// A version with a client, the library [`LIBRARY`] and one asset of its own, all served by
/// `client`.
fn version(client: &FakeClient, id: &str, asset_hash: &str) -> Arc<vanilla::VersionInfo> {
    version_from(client, id, asset_hash, |_| {})
}

/// [`version`], with `adjust` applied to its JSON.
fn version_from(
    client: &FakeClient,
    id: &str,
    asset_hash: &str,
    adjust: impl FnOnce(&mut serde_json::Value),
) -> Arc<vanilla::VersionInfo> {
    client.serve(&client_url(id), format!("client {id}"));
    client.serve(LIBRARY_URL, "library");
    client.serve(
        &assets_index_url(id),
        serde_json::to_vec(&serde_json::json!({
            "objects": { format!("sounds/{id}.ogg"): { "hash": asset_hash, "size": 5 } }
        }))
        .unwrap(),
    );
    client.serve(&asset_url(asset_hash), "sound");

    let mut json = serde_json::json!({
        "assetIndex": {
            "id": id, "sha1": "0", "size": 1, "totalSize": 5, "url": assets_index_url(id)
        },
        "assets": id,
        "downloads": { "client": { "sha1": "0", "size": 9, "url": client_url(id) } },
        "id": id,
        "javaVersion": null,
        "libraries": [{
            "name": LIBRARY,
            "downloads": { "artifact": {
                "path": "com/example/lib/1.0/lib-1.0.jar", "sha1": "0", "size": 7,
                "url": LIBRARY_URL
            } }
        }],
        "mainClass": "Main",
        "minimumLauncherVersion": 1,
        "releaseTime": "2020-01-01T00:00:00Z",
        "time": "2020-01-01T00:00:00Z",
        "type": "release"
    });
    adjust(&mut json);
    Arc::new(serde_json::from_value(json).unwrap())
}

const NATIVE_LIBRARY: &str = "com.example:native:1.0";
const NATIVE_URL: &str = "https://libraries.example/native-1.0-natives.jar";
const NATIVE_JAR_URL: &str = "https://libraries.example/native-1.0.jar";

/// Adds a library with natives for every OS, served from [`NATIVE_URL`].
fn with_natives(json: &mut serde_json::Value) {
    let classifier = serde_json::json!({
        "path": "n.jar", "sha1": "0", "size": 3, "url": NATIVE_URL
    });
    json["libraries"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "name": NATIVE_LIBRARY,
            "natives": {
                "linux": "natives-all", "windows": "natives-all", "osx": "natives-all"
            },
            "downloads": {
                "artifact": {
                    "path": "com/example/native/1.0/native-1.0.jar", "sha1": "0", "size": 3,
                    "url": NATIVE_JAR_URL
                },
                "classifiers": { "natives-all": classifier }
            }
        }));
}

fn natives_zip() -> Vec<u8> {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    writer
        .start_file("native.so", zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"machine code").unwrap();
    writer.finish().unwrap().into_inner()
}

fn library_path(location_info: &LocationInfo) -> std::path::PathBuf {
    location_info
        .libraries_dir()
        .join("com/example/lib/1.0/lib-1.0.jar")
}

/// Download `version`, waiting with `waiting`, in a task of its own; the bar comes back with
/// the result.
fn download(
    service: &Arc<Service>,
    version: &Arc<vanilla::VersionInfo>,
    waiting: Option<ProgressBarId>,
) -> tokio::task::JoinHandle<(Result<(), MinecraftDomainError>, Option<ProgressBarId>)> {
    let (service, version) = (service.clone(), version.clone());
    tokio::spawn(async move {
        let result = service
            .download_minecraft(&version, "x86_64", false, false, waiting.as_ref())
            .await;
        (result, waiting)
    })
}

/// A game without its natives starts and crashes: a failed download of them is the install's.
#[tokio::test]
async fn should_fail_the_install_when_the_natives_cannot_be_downloaded() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.20.1", ASSET_HASH, with_natives);
    fixture.client.serve(NATIVE_JAR_URL, "native jar");

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    assert!(
        matches!(result, Err(MinecraftDomainError::StorageFailure { .. })),
        "{result:?}"
    );
}

#[tokio::test]
async fn should_fail_the_install_when_the_natives_are_not_an_archive() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.20.1", ASSET_HASH, with_natives);
    fixture.client.serve(NATIVE_JAR_URL, "native jar");
    fixture.client.serve(NATIVE_URL, "not a zip");

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    assert!(
        matches!(result, Err(MinecraftDomainError::StorageFailure { .. })),
        "{result:?}"
    );
}

#[tokio::test]
async fn should_extract_the_natives_when_they_are_downloaded() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.20.1", ASSET_HASH, with_natives);
    fixture.client.serve(NATIVE_JAR_URL, "native jar");
    fixture.client.serve(NATIVE_URL, natives_zip());

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    result.unwrap();
    let extracted = fixture
        .location_info
        .version_natives_dir("1.20.1")
        .join("native.so");
    assert_eq!(std::fs::read(extracted).unwrap(), b"machine code");
}

/// A digest that is none of the bodies the tests serve.
const WRONG_SHA1: &str = "0123456789abcdef0123456789abcdef01234567";

#[tokio::test]
async fn should_refuse_the_client_when_its_digest_does_not_match() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.20.1", ASSET_HASH, |json| {
        json["downloads"]["client"]["sha1"] = WRONG_SHA1.into();
    });

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    assert!(
        matches!(result, Err(MinecraftDomainError::StorageFailure { .. })),
        "{result:?}"
    );
}

#[tokio::test]
async fn should_keep_the_client_when_its_digest_matches() {
    let fixture = fixture();
    let digest = sha1_async(Bytes::from_static(b"client 1.20.1"))
        .await
        .unwrap();
    let version = version_from(&fixture.client, "1.20.1", ASSET_HASH, |json| {
        json["downloads"]["client"]["sha1"] = digest.into();
    });

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    result.unwrap();
}

#[tokio::test]
async fn should_leave_no_library_when_its_digest_does_not_match() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.20.1", ASSET_HASH, |json| {
        json["libraries"][0]["downloads"]["artifact"]["sha1"] = WRONG_SHA1.into();
    });

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    assert!(result.is_err(), "{result:?}");
    assert!(!library_path(&fixture.location_info).exists());
}

#[tokio::test]
async fn should_leave_no_asset_when_its_digest_does_not_match() {
    let fixture = fixture();
    // The name of an asset is its digest; the body served is not what it names.
    let hash = "ab0123456789abcdef0123456789abcdef012345";
    let version = version(&fixture.client, "1.20.1", hash);

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    assert!(result.is_err(), "{result:?}");
    assert!(!fixture.location_info.object_dir(hash).exists());
}

const LOG_CONFIG_URL: &str = "https://launcher.example/client-1.12.xml";

/// The client log config, as the versions that have one name it.
fn with_log_config(json: &mut serde_json::Value) {
    json["logging"] = serde_json::json!({
        "client": {
            "argument": "-Dlog4j.configurationFile=${path}",
            "file": {
                "id": "client-1.12.xml", "sha1": "0", "size": 5, "url": LOG_CONFIG_URL
            },
            "type": "log4j2-xml"
        }
    });
}

fn log_config_path(location_info: &LocationInfo) -> std::path::PathBuf {
    location_info.log_configs_dir().join("client-1.12.xml")
}

#[tokio::test]
async fn should_download_the_log_config_the_version_names() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.12", ASSET_HASH, with_log_config);
    fixture.client.serve(LOG_CONFIG_URL, "<xml/>");

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    result.unwrap();
    assert_eq!(
        std::fs::read_to_string(log_config_path(&fixture.location_info)).unwrap(),
        "<xml/>"
    );
}

#[tokio::test]
async fn should_download_only_the_log_config_when_the_rest_is_there() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.12", ASSET_HASH, |_| {});
    let (result, _) = download(&fixture.service, &version, None).await.unwrap();
    result.unwrap();
    // The same game version, as the metadata names it now.
    let version = version_from(&fixture.client, "1.12", ASSET_HASH, with_log_config);
    fixture.client.serve(LOG_CONFIG_URL, "<xml/>");

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    result.unwrap();
    assert_eq!(fixture.client.fetches_of(&client_url("1.12")), 1);
    assert_eq!(fixture.client.fetches_of(LOG_CONFIG_URL), 1);
    assert!(log_config_path(&fixture.location_info).exists());
}

#[tokio::test]
async fn should_not_fetch_the_log_config_again_when_it_is_there() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.12", ASSET_HASH, with_log_config);
    fixture.client.serve(LOG_CONFIG_URL, "<xml/>");
    download(&fixture.service, &version, None)
        .await
        .unwrap()
        .0
        .unwrap();

    download(&fixture.service, &version, None)
        .await
        .unwrap()
        .0
        .unwrap();

    assert_eq!(fixture.client.fetches_of(LOG_CONFIG_URL), 1);
}

#[tokio::test]
async fn should_fail_the_install_when_the_log_config_cannot_be_downloaded() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.12", ASSET_HASH, with_log_config);

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    assert!(result.is_err(), "{result:?}");
    assert!(!log_config_path(&fixture.location_info).exists());
}

#[tokio::test]
async fn should_refuse_a_log_config_whose_id_leaves_its_directory() {
    let fixture = fixture();
    let version = version_from(&fixture.client, "1.12", ASSET_HASH, |json| {
        with_log_config(json);
        json["logging"]["client"]["file"]["id"] = "../escape.xml".into();
    });
    fixture.client.serve(LOG_CONFIG_URL, "<xml/>");

    let (result, _) = download(&fixture.service, &version, None).await.unwrap();

    assert!(result.is_err(), "{result:?}");
    assert!(
        !fixture
            .location_info
            .log_configs_dir()
            .parent()
            .unwrap()
            .join("escape.xml")
            .exists()
    );
}

async fn until_no_bars(storage: &InMemoryProgressBarStorage) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !storage.list().await.is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("bars were left behind");
}

#[tokio::test]
async fn should_download_a_version_once_and_mirror_it_for_both_installs_needing_it() {
    let mut fixture = fixture();
    let version = version(&fixture.client, "1.20.1", ASSET_HASH);
    fixture.client.gate(&client_url("1.20.1"));
    let a = fixture
        .progress
        .init_progress(pack("a"), 1.0, String::new())
        .await
        .unwrap();
    let b = fixture
        .progress
        .init_progress(pack("b"), 1.0, String::new())
        .await
        .unwrap();
    let (a_id, b_id) = (a.id, b.id);

    let first = download(&fixture.service, &version, Some(a));
    let [(_, shared)] = mirrors(&fixture.storage, 1).await[..] else {
        panic!("one mirror expected");
    };
    let second = download(&fixture.service, &version, Some(b));
    let mut expected = vec![(a_id, shared), (b_id, shared)];
    expected.sort_unstable();
    assert_eq!(mirrors(&fixture.storage, 2).await, expected);
    let shared_bar = fixture.storage.get(shared).await.unwrap().clone();
    assert_eq!(shared_bar.parent_id, None, "the download is nobody's");
    assert_eq!(
        shared_bar.progress_type,
        ProgressEventType::MinecraftVersionDownload {
            version: "1.20.1".into()
        }
    );

    fixture.client.open();
    let (first, a) = first.await.unwrap();
    let (second, b) = second.await.unwrap();
    first.unwrap();
    second.unwrap();
    for url in [
        client_url("1.20.1"),
        LIBRARY_URL.to_owned(),
        asset_url(ASSET_HASH),
    ] {
        assert_eq!(fixture.client.fetches_of(&url), 1, "{url}");
    }
    assert_eq!(
        std::fs::read_to_string(library_path(&fixture.location_info)).unwrap(),
        "library"
    );
    drop((a, b));
    until_no_bars(&fixture.storage).await;

    let events = sent(&mut fixture.events);
    let shared_bars: HashSet<_> = events
        .iter()
        .filter(|event| {
            matches!(
                event.event,
                ProgressEventType::MinecraftVersionDownload { .. }
            ) && event.mirror_of.is_none()
        })
        .map(|event| event.progress_bar_id)
        .collect();
    assert_eq!(
        shared_bars,
        HashSet::from([shared]),
        "one shared bar in all"
    );

    let mut children: Vec<_> = events
        .iter()
        .filter(|event| event.parent_id == Some(shared))
        .map(|event| (event.progress_bar_id, event.event.clone()))
        .collect::<HashMap<_, _>>()
        .into_values()
        .map(|event| format!("{event:?}"))
        .collect();
    children.sort();
    assert_eq!(
        children,
        [
            "FileDownload { file_name: \"1.20.1.jar\" }",
            "FileDownload { file_name: \"lib-1.0.jar\" }",
            "MinecraftAssetsDownload { version: \"1.20.1\", file_count: 1 }",
        ],
        "the client and each library by file, the assets as one"
    );

    let mirrors_ended: Vec<_> = events
        .iter()
        .filter(|event| event.mirror_of == Some(shared) && event.status != ProgressStatus::Running)
        .map(|event| event.status.clone())
        .collect();
    assert_eq!(mirrors_ended, [ProgressStatus::Done, ProgressStatus::Done]);
}

#[tokio::test]
async fn should_start_no_bar_and_no_mirror_when_the_version_is_downloaded_already() {
    let fixture = fixture();
    let version = version(&fixture.client, "1.20.1", ASSET_HASH);
    let (done, _) = download(&fixture.service, &version, None).await.unwrap();
    done.unwrap();
    until_no_bars(&fixture.storage).await;
    let fetched = fixture.client.fetched.lock().unwrap().len();
    let a = fixture
        .progress
        .init_progress(pack("a"), 1.0, String::new())
        .await
        .unwrap();
    let a_id = a.id;

    let (done, a) = download(&fixture.service, &version, Some(a)).await.unwrap();
    done.unwrap();

    let bars: Vec<_> = fixture
        .storage
        .list()
        .await
        .into_iter()
        .map(|bar| bar.id)
        .collect();
    assert_eq!(bars, [a_id], "nothing to wait on");
    assert_eq!(fixture.client.fetched.lock().unwrap().len(), fetched);
    drop(a);
}

/// Two game versions sharing a library: whichever install gets to it first writes it, and the
/// other waits for the whole file rather than taking a missing or partial one for present.
#[tokio::test]
async fn should_make_a_second_install_wait_for_a_shared_library_being_written() {
    let fixture = fixture();
    let first_version = version(&fixture.client, "1.20.1", ASSET_HASH);
    let second_version = version(&fixture.client, "1.20.2", "cd0123456789");
    fixture.client.gate(LIBRARY_URL);

    let first = download(&fixture.service, &first_version, None);
    let second = download(&fixture.service, &second_version, None);
    tokio::time::timeout(Duration::from_secs(5), async {
        while fixture.client.fetches_of(LIBRARY_URL) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    // Everything else of both versions is in; the library holds both up.
    tokio::time::timeout(Duration::from_secs(5), async {
        while fixture.client.fetches_of(&asset_url("cd0123456789")) == 0
            || fixture.client.fetches_of(&asset_url(ASSET_HASH)) == 0
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(!first.is_finished() && !second.is_finished());
    assert!(!library_path(&fixture.location_info).exists());

    fixture.client.open();
    first.await.unwrap().0.unwrap();
    second.await.unwrap().0.unwrap();
    assert_eq!(fixture.client.fetches_of(LIBRARY_URL), 1);
    let library_dir = library_path(&fixture.location_info);
    assert_eq!(std::fs::read_to_string(&library_dir).unwrap(), "library");
    let left: Vec<_> = std::fs::read_dir(library_dir.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(left, ["lib-1.0.jar"], "no temporary file is left");
}
