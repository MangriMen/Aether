//! `ModrinthContentProvider` on the resolve contract, against a fake HTTP client serving the
//! Modrinth API and CDN from memory: what plan a mod and a modpack become, and which requests
//! that takes.

use std::{
    collections::HashMap,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::{Value, json};
use sha2::{Digest, Sha512};
use tempfile::TempDir;

use crate::{
    features::{
        events::ProgressBarId,
        instance::{
            ContentIdentity, ContentMetadata, ContentProvider, ContentSource, ContentType,
            ContentVersionMetadata, FileSide, InstallTarget, InstanceError, InstanceGame,
            OverrideLocation, OverrideSource, PackInfo, PackSource, ResolveInstallPlanParams,
            ResolveTarget, infra::ModrinthContentProvider,
        },
        minecraft::{LoaderVersionPreference, ModLoader},
        settings::LocationInfo,
    },
    shared::{
        hash::FileHash,
        limits::ContentInstallLimits,
        request_client::{
            BytesProgress, FileDownload, Request, RequestClient, RequestError, SavedDownload,
        },
    },
};

const API: &str = "https://api.modrinth.com/v2";
const API_V3: &str = "https://api.modrinth.com/v3";
const SODIUM_URL: &str = "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d/sodium.jar";
const PACK_URL: &str = "https://cdn.modrinth.com/data/PACKID00/versions/PACKV002/pack.mrpack";

/// Serves bodies by URL (query ignored) and records every URL asked for.
#[derive(Default)]
struct FakeHttp {
    bodies: Mutex<HashMap<String, Bytes>>,
    requests: Mutex<Vec<String>>,
    /// The URLs asked for, query included.
    full_requests: Mutex<Vec<String>>,
}

impl FakeHttp {
    fn serve(&self, url: &str, body: impl Into<Bytes>) {
        self.bodies.lock().unwrap().insert(url.into(), body.into());
    }

    fn serve_json(&self, url: &str, body: &Value) {
        self.serve(url, serde_json::to_vec(body).unwrap());
    }

    fn body(&self, request: &Request) -> Result<Bytes, RequestError> {
        let url = request.url.split('?').next().unwrap().to_owned();
        self.requests.lock().unwrap().push(url.clone());
        self.full_requests.lock().unwrap().push(request.url.clone());
        self.bodies
            .lock()
            .unwrap()
            .get(&url)
            .cloned()
            .ok_or_else(|| RequestError::Middleware(anyhow::anyhow!("404 {url}")))
    }

    /// The ids of each `GET /projects` asked for, in the order asked.
    fn projects_batches(&self) -> Vec<Vec<String>> {
        self.batches(&format!("{API}/projects"))
    }

    /// The ids of each request to `route` that takes many ids at once, in the order asked.
    fn batches(&self, route: &str) -> Vec<Vec<String>> {
        self.full_requests
            .lock()
            .unwrap()
            .iter()
            .filter(|url| url.starts_with(&format!("{route}?")))
            .map(|url| {
                let url = url::Url::parse(url).unwrap();
                let (_, ids) = url.query_pairs().find(|(key, _)| key == "ids").unwrap();
                serde_json::from_str(&ids).unwrap()
            })
            .collect()
    }

    fn requests_to(&self, url: &str) -> usize {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|requested| *requested == url)
            .count()
    }
}

#[async_trait]
impl RequestClient for FakeHttp {
    async fn fetch_bytes(&self, request: Request) -> Result<Bytes, RequestError> {
        self.body(&request)
    }

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        _: Option<(&ProgressBarId, f64)>,
    ) -> Result<Bytes, RequestError> {
        self.body(&request)
    }

    async fn download_to_file(
        &self,
        request: Request,
        destination: &Path,
        max_size: u64,
        _: Option<BytesProgress<'_>>,
    ) -> Result<u64, RequestError> {
        let body = self.body(&request)?;
        // As the real client: a body past the limit is refused and nothing is left behind.
        if body.len() as u64 > max_size {
            return Err(RequestError::TooLarge { limit: max_size });
        }
        if let Some(hash) = &request.hash {
            assert!(hash.matches(&sha512_hex(&body)), "hash of {}", request.url);
        }
        std::fs::write(destination, &body).unwrap();
        Ok(body.len() as u64)
    }
    async fn download_to_file_conditional(
        &self,
        request: Request,
        destination: &Path,
        max_size: u64,
    ) -> Result<FileDownload, RequestError> {
        let size = self
            .download_to_file(request, destination, max_size, None)
            .await?;
        Ok(FileDownload::Saved(SavedDownload {
            size,
            ..SavedDownload::default()
        }))
    }
}

struct Fixture {
    _dir: TempDir,
    location_info: Arc<LocationInfo>,
    http: Arc<FakeHttp>,
    provider: ModrinthContentProvider<FakeHttp>,
}

impl Fixture {
    fn new() -> Self {
        Self::with_max_mrpack_bytes(ContentInstallLimits::default().max_mrpack_bytes)
    }

    fn with_max_mrpack_bytes(max_mrpack_bytes: u64) -> Self {
        let dir = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            dir.path().join("settings"),
            dir.path().to_path_buf(),
        ));
        let http = Arc::new(FakeHttp::default());
        let provider = ModrinthContentProvider::new(
            location_info.clone(),
            None,
            http.clone(),
            max_mrpack_bytes,
        );

        Self {
            _dir: dir,
            location_info,
            http,
            provider,
        }
    }

    fn serve_sodium(&self) {
        self.http.serve_json(
            &format!("{API}/project/sodium"),
            &project("AANobbMI", "sodium", "mod"),
        );
        self.http.serve_json(
            &format!("{API}/project/AANobbMI/version"),
            &json!([version(
                "IZskON6d",
                "AANobbMI",
                &["1.21.1"],
                &["fabric"],
                file(SODIUM_URL, "sodium.jar", b"sodium"),
            )]),
        );
    }

    /// A pack whose newest version is `PACKV002`, its `.mrpack` built and served.
    fn serve_pack(&self) {
        let mrpack = mrpack();
        self.http.serve_json(
            &format!("{API}/project/pack"),
            &project("PACKID00", "pack", "modpack"),
        );
        self.http.serve_json(
            &format!("{API}/project/PACKID00/version"),
            &json!([version(
                "PACKV002",
                "PACKID00",
                &["1.21.1"],
                &["neoforge"],
                file(PACK_URL, "pack.mrpack", &mrpack),
            )]),
        );
        self.http.serve(PACK_URL, mrpack);
    }
}

fn sha512_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    Sha512::digest(bytes)
        .iter()
        .fold(String::new(), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

fn project(id: &str, slug: &str, project_type: &str) -> Value {
    json!({
        "id": id, "slug": slug, "title": slug.to_uppercase(), "description": "desc",
        "categories": [], "client_side": "required", "server_side": "optional", "body": "",
        "status": "approved", "additional_categories": [], "donation_urls": [],
        "project_type": project_type, "downloads": 0, "icon_url": "https://cdn.modrinth.com/icon.png",
        "thread_id": "t", "monetization_status": "monetized", "team": "team",
        "published": "2024-01-01T00:00:00Z", "updated": "2024-01-01T00:00:00Z", "followers": 0,
        "license": { "id": "MIT", "name": "MIT" }, "versions": [], "game_versions": [],
        "loaders": [], "gallery": []
    })
}

fn file(url: &str, filename: &str, body: &[u8]) -> Value {
    json!({
        "hashes": { "sha1": "a".repeat(40), "sha512": sha512_hex(body).to_uppercase() },
        "url": url, "filename": filename, "primary": true, "size": body.len()
    })
}

fn version(
    id: &str,
    project_id: &str,
    game_versions: &[&str],
    loaders: &[&str],
    file: Value,
) -> Value {
    json!({
        "id": id, "project_id": project_id, "author_id": "author", "name": format!("Version {id}"),
        "version_number": "2.0.0", "dependencies": [], "game_versions": game_versions,
        "version_type": "release", "loaders": loaders, "featured": false, "status": "listed",
        "date_published": "2024-01-01T00:00:00Z", "downloads": 0, "files": Value::Array(vec![file])
    })
}

/// A `.mrpack`: two files in its index and one override.
fn mrpack() -> Vec<u8> {
    let index = json!({
        "formatVersion": 1, "game": "minecraft", "versionId": "2.0.0", "name": "Pack",
        "files": [
            {
                "path": "mods/sodium.jar", "hashes": { "sha1": "a".repeat(40), "sha512": "c".repeat(128) },
                "downloads": [SODIUM_URL], "fileSize": 6
            },
            {
                "path": "mods/server-only.jar", "hashes": { "sha1": "a".repeat(40), "sha512": "d".repeat(128) },
                "env": { "client": "unsupported", "server": "required" },
                "downloads": ["https://github.com/o/r/releases/download/v1/server-only.jar"], "fileSize": 1
            }
        ],
        "dependencies": { "minecraft": "1.21.4", "neoforge": "21.4.10" }
    });

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("modrinth.index.json", options).unwrap();
    zip.write_all(&serde_json::to_vec(&index).unwrap()).unwrap();
    zip.start_file("overrides/config/a.toml", options).unwrap();
    zip.write_all(b"a = 1").unwrap();
    zip.finish().unwrap().into_inner()
}

fn existing_fabric() -> ResolveTarget {
    ResolveTarget::Existing {
        instance_id: "instance".into(),
        game_version: "1.21.1".into(),
        loader: Some("fabric".into()),
    }
}

fn params(content_id: &str, target: ResolveTarget) -> ResolveInstallPlanParams {
    ResolveInstallPlanParams {
        content_id: content_id.into(),
        version: None,
        installed_version: None,
        target,
    }
}

#[tokio::test]
async fn should_plan_a_mod_as_its_file_with_its_identity_when_resolving_a_mod() {
    let fixture = Fixture::new();
    fixture.serve_sodium();

    let plan = fixture
        .provider
        .resolve_install_plan(params("sodium", existing_fabric()))
        .await
        .unwrap();

    assert_eq!(plan.target, InstallTarget::existing("instance"));
    assert!(plan.overrides.is_empty());
    let [entry] = &plan.files[..] else {
        panic!("one file expected: {:?}", plan.files)
    };
    assert_eq!(entry.relative_path, "mods/sodium.jar");
    assert_eq!(entry.content_type, ContentType::Mod);
    assert_eq!(entry.size, Some(6));
    // The version's name, as the older install path recorded it.
    assert_eq!(entry.name.as_deref(), Some("Version IZskON6d"));
    assert_eq!(
        entry.source,
        ContentSource::Direct {
            url: SODIUM_URL.into(),
            hash: Some(FileHash::Sha512(sha512_hex(b"sodium"))),
            identity: Some(ContentIdentity {
                platform: "modrinth".into(),
                content_id: "AANobbMI".into(),
                version: "IZskON6d".into(),
            }),
        }
    );
    // Resolving downloads nothing: the file is the executor's to fetch.
    assert_eq!(fixture.http.requests_to(SODIUM_URL), 0);
}

#[tokio::test]
async fn should_refuse_a_mod_when_the_target_is_a_new_instance() {
    let fixture = Fixture::new();
    fixture.serve_sodium();

    let result = fixture
        .provider
        .resolve_install_plan(params("sodium", ResolveTarget::New))
        .await;

    assert!(matches!(
        result,
        Err(InstanceError::ContentProviderError { .. })
    ));
}

#[tokio::test]
async fn should_plan_a_modpack_from_its_mrpack_when_the_target_is_a_new_instance() {
    let fixture = Fixture::new();
    fixture.serve_pack();

    let plan = fixture
        .provider
        .resolve_install_plan(params("pack", ResolveTarget::New))
        .await
        .unwrap();

    let InstallTarget::New(new_instance) = &plan.target else {
        panic!("a new instance expected: {:?}", plan.target)
    };
    assert_eq!(new_instance.name, "Pack");
    assert_eq!(new_instance.game_version, "1.21.4");
    assert_eq!(new_instance.mod_loader, ModLoader::NeoForge);
    assert_eq!(
        new_instance.loader_version,
        Some(LoaderVersionPreference::Exact("21.4.10".into()))
    );
    assert_eq!(
        new_instance.pack_info,
        Some(PackInfo {
            provider_id: ModrinthContentProvider::provider_id(),
            modpack_id: "PACKID00".into(),
            version_id: "PACKV002".into(),
        })
    );

    let paths: Vec<_> = plan
        .files
        .iter()
        .map(|f| f.relative_path.as_str())
        .collect();
    assert_eq!(paths, ["mods/sodium.jar", "mods/server-only.jar"]);
    assert_eq!(plan.files[1].side, FileSide::Server);
    assert!(matches!(
        &plan.files[0].source,
        ContentSource::Direct { identity: Some(identity), .. } if identity.content_id == "AANobbMI"
    ));

    let [
        OverrideSource {
            location:
                OverrideLocation::Archive {
                    archive_path,
                    subdir,
                },
            side: FileSide::Both,
        },
        ..,
    ] = &plan.overrides[..]
    else {
        panic!("archive overrides expected: {:?}", plan.overrides)
    };
    assert_eq!(subdir, "overrides");
    assert!(archive_path.starts_with(fixture.location_info.cache_dir()));
    assert!(archive_path.is_file());
    // The sides' own directories are layered over it, from the same archive.
    assert_eq!(plan.overrides.len(), 3);
}

#[tokio::test]
async fn should_refuse_a_modpack_when_its_mrpack_is_larger_than_the_limit() {
    let limit = mrpack().len() as u64 - 1;
    let fixture = Fixture::with_max_mrpack_bytes(limit);
    fixture.serve_pack();

    let err = fixture
        .provider
        .resolve_install_plan(params("pack", ResolveTarget::New))
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::ContentDownloadError(reason) if reason.contains(&limit.to_string())),
        "{err:?}"
    );
    let cache = fixture.location_info.cache_dir().join("modrinth/mrpack");
    assert!(!cache.exists() || std::fs::read_dir(cache).unwrap().next().is_none());
}

fn age(path: &std::path::Path, days: u64) {
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(days * 86_400))
        .unwrap();
}

/// The archive a plan points at survives the cleaning of the cache, however old it was; an
/// archive nobody used for long goes, and nothing outside the archive directory is touched.
#[tokio::test]
async fn should_clear_stale_archives_when_the_plan_points_at_a_fresh_one() {
    let fixture = Fixture::new();
    fixture.serve_pack();
    let plan = || {
        fixture
            .provider
            .resolve_install_plan(params("pack", ResolveTarget::New))
    };

    let first = plan().await.unwrap();
    let OverrideLocation::Archive { archive_path, .. } = &first.overrides[0].location else {
        panic!("archive overrides expected")
    };
    let dir = archive_path.parent().unwrap();
    let stale = dir.join("stale.mrpack");
    std::fs::write(&stale, b"old").unwrap();
    age(&stale, 30);
    let elsewhere = fixture
        .location_info
        .cache_dir()
        .join("modrinth")
        .join("old.mrpack");
    std::fs::write(&elsewhere, b"not ours").unwrap();
    age(&elsewhere, 30);
    age(archive_path, 30);

    let second = plan().await.unwrap();

    assert_eq!(second.overrides, first.overrides);
    assert!(archive_path.is_file());
    assert!(!stale.exists());
    assert!(elsewhere.is_file());
    // Served from the cache: not downloaded a second time.
    assert_eq!(fixture.http.requests_to(PACK_URL), 1);
}

/// A pack whose newest version is a beta (`PACKV003`), after a release (`PACKV002`).
fn serve_pack_with_newer_beta(fixture: &Fixture) {
    let mrpack = mrpack();
    let mut beta = version(
        "PACKV003",
        "PACKID00",
        &["1.21.1"],
        &["neoforge"],
        file(PACK_URL, "pack.mrpack", &mrpack),
    );
    beta["version_type"] = json!("beta");
    beta["date_published"] = json!("2024-06-01T00:00:00Z");
    fixture.http.serve_json(
        &format!("{API}/project/pack"),
        &project("PACKID00", "pack", "modpack"),
    );
    fixture.http.serve_json(
        &format!("{API}/project/PACKID00/version"),
        &json!([
            beta,
            version(
                "PACKV002",
                "PACKID00",
                &["1.21.1"],
                &["neoforge"],
                file(PACK_URL, "pack.mrpack", &mrpack),
            )
        ]),
    );
    fixture.http.serve(PACK_URL, mrpack);
}

/// Installed without a version asked for, a pack gets its newest release, as a mod does.
#[tokio::test]
async fn should_install_the_newest_release_of_a_pack_when_a_newer_beta_exists() {
    let fixture = Fixture::new();
    serve_pack_with_newer_beta(&fixture);

    let plan = fixture
        .provider
        .resolve_install_plan(params("pack", ResolveTarget::New))
        .await
        .unwrap();

    let InstallTarget::New(new_instance) = plan.target else {
        panic!("a new instance expected")
    };
    assert_eq!(new_instance.pack_info.unwrap().version_id, "PACKV002");
}

async fn describe_pack_update(fixture: &Fixture, installed: &str) -> Option<String> {
    fixture
        .provider
        .resolve_pack_metadata(PackSource::Content {
            content_id: "pack".into(),
            version: None,
            installed_version: Some(installed.into()),
        })
        .await
        .unwrap()
        .version_id
}

/// A check for updates of a release does not offer a newer beta: the installed release is the
/// answer, the same id as installed — no update.
#[tokio::test]
async fn should_not_offer_a_beta_when_the_installed_version_is_a_release() {
    let fixture = Fixture::new();
    serve_pack_with_newer_beta(&fixture);

    let offered = describe_pack_update(&fixture, "PACKV002").await;

    assert_eq!(offered.as_deref(), Some("PACKV002"));
}

/// A beta of a pack follows betas, and is not rolled back to the older release.
#[tokio::test]
async fn should_keep_the_newest_beta_of_a_pack_when_the_newest_release_is_older() {
    let fixture = Fixture::new();
    serve_pack_with_newer_beta(&fixture);

    let offered = describe_pack_update(&fixture, "PACKV003").await;

    assert_eq!(offered.as_deref(), Some("PACKV003"));
}

/// A mod version as the API lists it, of a kind and a publication day of 2024.
fn mod_version(id: &str, game_version: &str, version_type: &str, day: u32) -> Value {
    let mut version = version(
        id,
        "AANobbMI",
        &[game_version],
        &["fabric"],
        file(SODIUM_URL, "sodium.jar", b"sodium"),
    );
    version["version_type"] = json!(version_type);
    version["date_published"] = json!(format!("2024-01-{day:02}T00:00:00Z"));
    version
}

async fn resolve_sodium(versions: Value, installed: &str) -> String {
    let fixture = Fixture::new();
    fixture.http.serve_json(
        &format!("{API}/project/sodium"),
        &project("AANobbMI", "sodium", "mod"),
    );
    fixture
        .http
        .serve_json(&format!("{API}/project/AANobbMI/version"), &versions);

    let plan = fixture
        .provider
        .resolve_install_plan(ResolveInstallPlanParams {
            installed_version: Some(installed.into()),
            ..params("sodium", existing_fabric())
        })
        .await
        .unwrap();
    let [entry] = &plan.files[..] else {
        panic!("one file expected: {:?}", plan.files)
    };
    let ContentSource::Direct {
        identity: Some(identity),
        ..
    } = &entry.source
    else {
        panic!(
            "a direct file with its identity expected: {:?}",
            entry.source
        )
    };

    identity.version.clone()
}

/// Reinstalled without a version, a mod on a beta is not rolled back to the older release.
#[tokio::test]
async fn should_keep_a_mod_on_its_beta_when_the_newest_release_is_older() {
    let versions = json!([
        mod_version("RELEASE1", "1.21.1", "release", 1),
        mod_version("BETA0002", "1.21.1", "beta", 2),
    ]);

    assert_eq!(resolve_sodium(versions, "BETA0002").await, "BETA0002");
}

#[tokio::test]
async fn should_move_a_mod_from_its_beta_when_a_newer_release_exists() {
    let versions = json!([
        mod_version("BETA0001", "1.21.1", "beta", 1),
        mod_version("RELEASE2", "1.21.1", "release", 2),
        mod_version("RELEASE0", "1.21.1", "release", 0),
    ]);

    assert_eq!(resolve_sodium(versions, "BETA0001").await, "RELEASE2");
}

/// An installed version made for another game version cannot stay: the newest fitting version
/// of its channel is taken, even an older one.
#[tokio::test]
async fn should_replace_an_installed_mod_when_it_no_longer_fits_the_game() {
    let versions = json!([
        mod_version("BETA0001", "1.21.1", "beta", 1),
        mod_version("RELEASE0", "1.21.1", "release", 0),
        mod_version("BETA0009", "1.20.1", "beta", 9),
    ]);

    assert_eq!(resolve_sodium(versions, "BETA0009").await, "BETA0001");
}

/// An update of the pack: the plan moves the instance to the game of the new version, and the
/// archive already in the cache is not downloaded again.
#[tokio::test]
async fn should_move_the_instance_to_the_game_of_the_pack_when_the_pack_is_updated() {
    let fixture = Fixture::new();
    fixture.serve_pack();
    fixture
        .provider
        .resolve_install_plan(params("pack", ResolveTarget::New))
        .await
        .unwrap();

    let plan = fixture
        .provider
        .resolve_install_plan(params("pack", existing_fabric()))
        .await
        .unwrap();

    assert_eq!(
        plan.target,
        InstallTarget::Existing {
            instance_id: "instance".into(),
            game: Some(InstanceGame {
                game_version: "1.21.4".into(),
                mod_loader: ModLoader::NeoForge,
                loader_version: Some(LoaderVersionPreference::Exact("21.4.10".into())),
            }),
        }
    );
    assert_eq!(fixture.http.requests_to(PACK_URL), 1);
}

#[tokio::test]
async fn should_resolve_a_ref_to_the_primary_file_when_the_version_has_several() {
    let fixture = Fixture::new();
    fixture.http.serve_json(
        &format!("{API}/version/IZskON6d"),
        &version(
            "IZskON6d",
            "AANobbMI",
            &["1.21.1"],
            &["fabric"],
            file(SODIUM_URL, "sodium.jar", b"sodium"),
        ),
    );

    let resolved = fixture
        .provider
        .resolve_file("AANobbMI", Some("IZskON6d"), "1.21.1", Some("fabric"))
        .await
        .unwrap();

    assert_eq!(resolved.version, "IZskON6d");
    assert_eq!(resolved.url, SODIUM_URL);
    assert_eq!(resolved.file_name, "sodium.jar");
    assert_eq!(resolved.hash, Some(FileHash::Sha512(sha512_hex(b"sodium"))));
    assert_eq!(resolved.name.as_deref(), Some("Version IZskON6d"));
}

/// A slug names the project as well as its id does.
#[tokio::test]
async fn should_resolve_a_ref_to_a_version_of_its_project_when_given_a_slug() {
    let fixture = Fixture::new();
    fixture.serve_sodium();
    fixture.http.serve_json(
        &format!("{API}/version/IZskON6d"),
        &version(
            "IZskON6d",
            "AANobbMI",
            &["1.21.1"],
            &["fabric"],
            file(SODIUM_URL, "sodium.jar", b"sodium"),
        ),
    );

    let resolved = fixture
        .provider
        .resolve_file("sodium", Some("IZskON6d"), "1.21.1", Some("fabric"))
        .await
        .unwrap();

    assert_eq!(resolved.version, "IZskON6d");
}

/// A version of another project would be recorded as the wrong content.
#[tokio::test]
async fn should_refuse_a_version_when_it_belongs_to_another_project() {
    let fixture = Fixture::new();
    fixture.http.serve_json(
        &format!("{API}/version/IZskON6d"),
        &version(
            "IZskON6d",
            "AANobbMI",
            &["1.21.1"],
            &["fabric"],
            file(SODIUM_URL, "sodium.jar", b"sodium"),
        ),
    );

    fixture.http.serve_json(
        &format!("{API}/project/P7dR8mSH"),
        &project("P7dR8mSH", "lithium", "mod"),
    );

    let result = fixture
        .provider
        .resolve_file("P7dR8mSH", Some("IZskON6d"), "1.21.1", Some("fabric"))
        .await;

    assert!(matches!(
        result,
        Err(InstanceError::ContentProviderError { .. })
    ));
}

/// A check for updates asks for the newest version: answered from the API, without
/// downloading the archive.
#[tokio::test]
async fn should_describe_a_pack_without_downloading_it_when_it_is_on_modrinth() {
    let fixture = Fixture::new();
    fixture.serve_pack();

    let metadata = fixture
        .provider
        .resolve_pack_metadata(PackSource::Content {
            content_id: "pack".into(),
            version: None,
            installed_version: None,
        })
        .await
        .unwrap();

    assert_eq!(metadata.version_id.as_deref(), Some("PACKV002"));
    assert_eq!(metadata.name, "PACK");
    assert_eq!(metadata.game_version, "1.21.1");
    assert_eq!(metadata.mod_loader, ModLoader::NeoForge);
    assert_eq!(fixture.http.requests_to(PACK_URL), 0);
}

#[tokio::test]
async fn should_describe_a_pack_from_its_index_when_given_a_url() {
    let fixture = Fixture::new();
    fixture.serve_pack();

    let metadata = fixture
        .provider
        .resolve_pack_metadata(PackSource::Url(PACK_URL.into()))
        .await
        .unwrap();

    assert_eq!(metadata.name, "Pack");
    assert_eq!(metadata.version_id.as_deref(), Some("PACKV002"));
    assert_eq!(metadata.loader_version.as_deref(), Some("21.4.10"));
    assert_eq!(metadata.file_count, Some(2));
}

fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|&id| id.to_owned()).collect()
}

#[tokio::test]
async fn should_describe_the_projects_modrinth_knows_when_asked_for_their_metadata() {
    let fixture = Fixture::new();
    let mut fabric_api = project("P7dR8mSH", "fabric-api", "mod");
    fabric_api["icon_url"] = Value::Null;
    fabric_api["description"] = json!("");
    fixture.http.serve_json(
        &format!("{API}/projects"),
        &json!([fabric_api, project("AANobbMI", "sodium", "mod")]),
    );
    fixture
        .http
        .serve_json(&format!("{API_V3}/teams"), &json!([]));

    let mut metadata = fixture
        .provider
        .get_contents_metadata(&ids(&["sodium", "P7dR8mSH", "unknown"]))
        .await
        .unwrap();
    metadata.sort_by(|a, b| a.content_id.cmp(&b.content_id));

    // Keyed by the project's id, even when asked by its slug.
    assert_eq!(
        metadata,
        [
            ContentMetadata {
                content_id: "AANobbMI".into(),
                slug: "sodium".into(),
                name: "SODIUM".into(),
                description: Some("desc".into()),
                author: None,
                icon_url: Some("https://cdn.modrinth.com/icon.png".into()),
            },
            ContentMetadata {
                content_id: "P7dR8mSH".into(),
                slug: "fabric-api".into(),
                name: "FABRIC-API".into(),
                description: None,
                author: None,
                icon_url: None,
            },
        ]
    );
    assert_eq!(
        fixture.http.projects_batches(),
        [ids(&["sodium", "P7dR8mSH", "unknown"])]
    );
}

#[tokio::test]
async fn should_ask_in_batches_and_list_each_project_once_when_given_many_ids() {
    let fixture = Fixture::new();
    // Every batch answers with the same project: asked by id and by slug, it comes back twice.
    fixture.http.serve_json(
        &format!("{API}/projects"),
        &json!([project("AANobbMI", "sodium", "mod")]),
    );
    fixture
        .http
        .serve_json(&format!("{API_V3}/teams"), &json!([]));
    let mut asked: Vec<String> = (0..250).map(|i| format!("id{i}")).collect();
    asked.extend(ids(&["id0", "id1"]));

    let metadata = fixture
        .provider
        .get_contents_metadata(&asked)
        .await
        .unwrap();

    assert_eq!(metadata.len(), 1);
    let batches = fixture.http.projects_batches();
    assert_eq!(batches.len(), 3);
    assert!(batches.iter().all(|batch| batch.len() <= 100));
    let mut all: Vec<String> = batches.into_iter().flatten().collect();
    all.sort();
    let mut expected: Vec<String> = (0..250).map(|i| format!("id{i}")).collect();
    expected.sort();
    assert_eq!(all, expected, "each id asked exactly once");
}

#[tokio::test]
async fn should_ask_nothing_when_given_no_ids() {
    let fixture = Fixture::new();

    let metadata = fixture.provider.get_contents_metadata(&[]).await.unwrap();

    assert!(metadata.is_empty());
    assert!(fixture.http.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_fail_when_a_batch_cannot_be_fetched() {
    let fixture = Fixture::new();

    let result = fixture
        .provider
        .get_contents_metadata(&ids(&["AANobbMI"]))
        .await;

    assert!(matches!(
        result,
        Err(InstanceError::ContentDownloadError(_))
    ));
}

fn team_member(team_id: &str, username: &str, is_owner: bool) -> Value {
    json!({
        "team_id": team_id, "role": "Developer", "is_owner": is_owner, "accepted": true,
        "ordering": 0, "user": { "id": format!("u-{username}"), "username": username, "name": null }
    })
}

#[tokio::test]
async fn should_name_the_owner_or_the_organization_as_author_when_asked_for_metadata() {
    let fixture = Fixture::new();
    let mut by_team = project("TEAMPROJ", "by-team", "mod");
    by_team["team"] = json!("team1");
    let mut by_organization = project("ORGPROJ0", "by-organization", "mod");
    by_organization["team"] = json!("team2");
    by_organization["organization"] = json!("org1");
    let mut no_team = project("NOTEAM00", "no-team", "mod");
    no_team["team"] = json!("gone");
    fixture.http.serve_json(
        &format!("{API}/projects"),
        &json!([by_team, by_organization, no_team]),
    );
    fixture.http.serve_json(
        &format!("{API_V3}/teams"),
        &json!([[
            team_member("team1", "helper", false),
            team_member("team1", "owner", true),
        ]]),
    );
    fixture.http.serve_json(
        &format!("{API_V3}/organizations"),
        &json!([{ "id": "org1", "slug": "org", "name": "Org Name", "team_id": "orgteam" }]),
    );

    let metadata = fixture
        .provider
        .get_contents_metadata(&ids(&["TEAMPROJ", "ORGPROJ0", "NOTEAM00"]))
        .await
        .unwrap();

    let authors: HashMap<_, _> = metadata
        .into_iter()
        .map(|item| (item.content_id, item.author))
        .collect();
    assert_eq!(authors["TEAMPROJ"].as_deref(), Some("owner"));
    assert_eq!(authors["ORGPROJ0"].as_deref(), Some("Org Name"));
    assert_eq!(authors["NOTEAM00"], None, "a team Modrinth did not list");
    // The team of a project an organization owns is not asked about.
    let mut teams = fixture.http.batches(&format!("{API_V3}/teams"));
    for batch in &mut teams {
        batch.sort();
    }
    assert_eq!(teams, [ids(&["gone", "team1"])]);
    assert_eq!(
        fixture.http.batches(&format!("{API_V3}/organizations")),
        [ids(&["org1"])]
    );
}

#[tokio::test]
async fn should_fail_when_the_owners_cannot_be_fetched() {
    let fixture = Fixture::new();
    fixture.http.serve_json(
        &format!("{API}/projects"),
        &json!([project("AANobbMI", "sodium", "mod")]),
    );

    let result = fixture
        .provider
        .get_contents_metadata(&ids(&["AANobbMI"]))
        .await;

    assert!(matches!(
        result,
        Err(InstanceError::ContentDownloadError(_))
    ));
}

fn version_metadata(id: &str, version_number: &str, name: &str) -> Value {
    json!({
        "id": id, "project_id": "AANobbMI", "author_id": "DzLrfrbK", "featured": false,
        "name": name, "version_number": version_number, "changelog": null,
        "changelog_url": null, "date_published": "2024-04-29T15:59:42.831851Z",
        "downloads": 1, "version_type": "release", "status": "listed",
        "requested_status": null, "game_versions": ["1.21.1"], "loaders": ["fabric"],
        "files": [], "dependencies": []
    })
}

#[tokio::test]
async fn should_describe_the_versions_modrinth_knows_when_asked_for_their_metadata() {
    let fixture = Fixture::new();
    fixture.http.serve_json(
        &format!("{API}/versions"),
        &json!([
            version_metadata("IZskON6d", "mc1.20.6-0.5.8", "Sodium 0.5.8"),
            version_metadata("NOTITLE0", "1.0.0", ""),
            // An entry of a shape not expected is left out, not the whole answer.
            { "id": "ODDONE00" },
        ]),
    );

    let mut metadata = fixture
        .provider
        .get_versions_metadata(&ids(&["IZskON6d", "NOTITLE0", "ODDONE00", "unknown"]))
        .await
        .unwrap();
    metadata.sort_by(|a, b| a.version_id.cmp(&b.version_id));

    assert_eq!(
        metadata,
        [
            ContentVersionMetadata {
                version_id: "IZskON6d".into(),
                version_number: "mc1.20.6-0.5.8".into(),
                name: Some("Sodium 0.5.8".into()),
            },
            ContentVersionMetadata {
                version_id: "NOTITLE0".into(),
                version_number: "1.0.0".into(),
                name: None,
            },
        ]
    );
    assert_eq!(
        fixture.http.batches(&format!("{API}/versions")),
        [ids(&["IZskON6d", "NOTITLE0", "ODDONE00", "unknown"])]
    );
}

#[tokio::test]
async fn should_ask_for_versions_in_batches_of_each_id_once_when_given_many() {
    let fixture = Fixture::new();
    fixture
        .http
        .serve_json(&format!("{API}/versions"), &json!([]));
    let mut asked: Vec<String> = (0..250).map(|i| format!("v{i}")).collect();
    asked.extend(ids(&["v0", "v1"]));

    let metadata = fixture
        .provider
        .get_versions_metadata(&asked)
        .await
        .unwrap();

    assert!(metadata.is_empty());
    let batches = fixture.http.batches(&format!("{API}/versions"));
    assert_eq!(batches.len(), 3);
    assert!(batches.iter().all(|batch| batch.len() <= 100));
    let mut all: Vec<String> = batches.into_iter().flatten().collect();
    all.sort();
    let mut expected: Vec<String> = (0..250).map(|i| format!("v{i}")).collect();
    expected.sort();
    assert_eq!(all, expected, "each id asked exactly once");
}

#[tokio::test]
async fn should_fail_when_a_batch_of_versions_cannot_be_fetched() {
    let fixture = Fixture::new();

    let empty = fixture.provider.get_versions_metadata(&[]).await.unwrap();
    let result = fixture
        .provider
        .get_versions_metadata(&ids(&["IZskON6d"]))
        .await;

    assert!(empty.is_empty());
    assert!(matches!(
        result,
        Err(InstanceError::ContentDownloadError(_))
    ));
}
