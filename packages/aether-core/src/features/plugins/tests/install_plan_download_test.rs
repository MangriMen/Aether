//! The download policy of the install plan executor end to end, against loopback servers.
//!
//! A URL a plugin supplied (in its plan, or by resolving a ref) reaches only that plugin's
//! `allowed_hosts`; a built-in provider's URL goes through the application's client. Whatever
//! fails, the instance is left as it was.

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::{
        events::{
            Event, EventEmitter, EventError, ProgressBarId, ProgressEventType, ProgressService,
        },
        instance::{
            CachedContentAuthor, CachedContentMetadata, CachedContentVersionMetadata,
            CapabilityMetadata, ChangeInstanceGameUseCasePort, ContentAuthor,
            ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentDownloader,
            ContentItem, ContentMetadata, ContentMetadataCache, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentSource, ContentType, ContentVersion, ContentVersionMetadata,
            CreateInstanceUseCasePort, InstallFileEntry, InstallPlan, InstallPlanExecutor,
            InstallTarget, Instance, InstanceBuilder, InstanceError, InstanceGame, InstanceId,
            InstanceStorage, NewInstance, PlanOrigin, ProviderId, RemoveInstanceUseCasePort,
            ResolvedFile,
            infra::{FsContentFileService, NoPlatformProviderSelection, SqlitePackStorage},
        },
        minecraft::ModLoader,
        plugins::{
            ApiConfig, LoadConfig, Plugin, PluginError, PluginManifest, PluginMetadata,
            PluginRegistry, PluginSettings, PluginSettingsStorage, PluginState, RuntimeConfig,
            infra::{
                ProviderContentDownloader,
                test_server::{TestServer, ok, redirect},
            },
        },
        settings::LocationInfo,
    },
    shared::{
        capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
        fetch::FetchSemaphore,
        hash::FileHash,
        request_client::infra::ReqwestClient,
    },
};

const PLUGIN_ID: &str = "cf-plugin";
const INSTANCE_ID: &str = "instance";
const BODY: &[u8] = b"mod bytes";

/// Two loopback addresses: two different hosts as far as `allowed_hosts` is concerned.
const ALLOWED: &str = "127.0.0.1";
const OTHER: &str = "127.0.0.2";

// ─── Mocks ─────────────────────────────────────────────────────

/// Does not keep anything.
struct NoMetadataCache;

#[async_trait]
impl ContentMetadataCache for NoMetadataCache {
    async fn get_many(
        &self,
        _: &ProviderId,
        _: &[String],
    ) -> HashMap<String, CachedContentMetadata> {
        HashMap::new()
    }

    async fn put_keyed_at(
        &self,
        _: &ProviderId,
        _: &[(String, ContentMetadata)],
        _: SystemTime,
    ) -> bool {
        true
    }

    async fn get_versions(
        &self,
        _: &ProviderId,
        _: &[String],
    ) -> HashMap<String, CachedContentVersionMetadata> {
        HashMap::new()
    }

    async fn put_versions(&self, _: &ProviderId, _: &[ContentVersionMetadata]) -> bool {
        true
    }

    async fn get_authors(
        &self,
        _: &ProviderId,
        _: &[String],
    ) -> HashMap<String, CachedContentAuthor> {
        HashMap::new()
    }

    async fn put_authors(&self, _: &ProviderId, _: &[ContentAuthor]) -> bool {
        true
    }
}

struct NoProgress;

#[async_trait]
impl ProgressService for NoProgress {
    async fn init_progress(
        &self,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn init_or_edit_progress(
        &self,
        _progress_bar_id: Option<ProgressBarId>,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn emit_progress(
        &self,
        _progress_bar_id: &ProgressBarId,
        _increment_frac: f64,
        _message: Option<&str>,
    ) -> Result<(), EventError> {
        Ok(())
    }
    async fn edit_progress(
        &self,
        _progress_bar_id: &ProgressBarId,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<(), EventError> {
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

#[derive(Default)]
struct MemSettings(Mutex<HashMap<String, PluginSettings>>);

#[async_trait]
impl PluginSettingsStorage for MemSettings {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        Ok(self.0.lock().unwrap().get(plugin_id).cloned())
    }

    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError> {
        self.0
            .lock()
            .unwrap()
            .insert(plugin_id.into(), settings.clone());
        Ok(())
    }
}

/// The plugin's content provider, serving `curseforge`: resolves every ref to `url`.
struct PluginProvider {
    metadata: ContentProviderCapabilityMetadata,
    url: String,
}

#[async_trait]
impl ContentProvider for PluginProvider {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.metadata
    }

    fn provider_id(&self) -> ProviderId {
        plugin_origin()
    }

    async fn search(&self, _: ContentSearchParams) -> Result<ContentSearchResult, InstanceError> {
        unimplemented!()
    }

    async fn get_content(&self, _: String) -> Result<ContentItem, InstanceError> {
        unimplemented!()
    }

    async fn list_versions(&self, _: String) -> Result<Vec<ContentVersion>, InstanceError> {
        unimplemented!()
    }

    async fn resolve_file(
        &self,
        _content_id: &str,
        _version: Option<&str>,
        _game_version: &str,
        _loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError> {
        Ok(ResolvedFile {
            version: "4512345".into(),
            url: self.url.clone(),
            hash: Some(sha1_of(BODY)),
            file_name: "jei.jar".into(),
            size: None,
            name: None,
        })
    }

    async fn check_compatibility(
        &self,
        _: &[Instance],
        _: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<InstanceId, ContentCompatibilityResult>, InstanceError> {
        unimplemented!()
    }
}

struct OneInstanceStorage(Instance);

#[async_trait]
impl InstanceStorage for OneInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(vec![self.0.clone()])
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        if id == self.0.id() {
            Ok(self.0.clone())
        } else {
            Err(InstanceError::NotFound {
                instance_id: id.to_string(),
            })
        }
    }

    async fn upsert(&self, _: &Instance) -> Result<(), InstanceError> {
        unimplemented!()
    }

    async fn update(
        &self,
        id: &InstanceId,
        change: crate::features::instance::InstanceChange<'_>,
    ) -> Result<Instance, InstanceError> {
        let mut instance = self.get(id).await?;
        if change(&mut instance)? {
            self.upsert(&instance).await?;
        }
        Ok(instance)
    }

    async fn remove(&self, _: &InstanceId) -> Result<(), InstanceError> {
        unimplemented!()
    }
}

struct NoCreateInstance;

#[async_trait]
impl CreateInstanceUseCasePort for NoCreateInstance {
    async fn execute(
        &self,
        _: NewInstance,
        _: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        unimplemented!("these plans install into an existing instance")
    }

    async fn execute_for_pack(
        &self,
        _: NewInstance,
        _: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        unimplemented!("these plans install into an existing instance")
    }
}

struct NoChangeInstanceGame;

#[async_trait]
impl ChangeInstanceGameUseCasePort for NoChangeInstanceGame {
    async fn execute(
        &self,
        _: InstanceId,
        _: InstanceGame,
        _: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        unimplemented!("these plans keep the game of the instance")
    }
}

struct NoRemoveInstance;

#[async_trait]
impl RemoveInstanceUseCasePort for NoRemoveInstance {
    async fn execute(&self, _: InstanceId) -> Result<(), InstanceError> {
        unimplemented!("these plans create no instance")
    }
}

// ─── Fixture ───────────────────────────────────────────────────

/// The servers live on loopback, which the production address filter blocks; what is under test
/// here is `allowed_hosts`, which the vetted client checks before any address.
fn allow_any_address(_ip: IpAddr) -> bool {
    false
}

fn sha1_of(bytes: &[u8]) -> FileHash {
    FileHash::Sha1(sha1_smol::Sha1::from(bytes).hexdigest())
}

fn plugin_origin() -> ProviderId {
    ProviderId {
        plugin_id: PLUGIN_ID.into(),
        capability_id: "curseforge".into(),
    }
}

fn builtin_origin() -> ProviderId {
    ProviderId {
        plugin_id: ProviderId::BUILTIN_PLUGIN_ID.into(),
        capability_id: "modrinth".into(),
    }
}

fn manifest(allowed_hosts: &[&str]) -> PluginManifest {
    PluginManifest {
        metadata: PluginMetadata {
            id: PLUGIN_ID.into(),
            name: "CurseForge".into(),
            version: semver::Version::new(0, 1, 0),
            description: None,
            authors: vec![],
            license: None,
        },
        runtime: RuntimeConfig {
            allowed_hosts: allowed_hosts.iter().map(|&host| host.into()).collect(),
            allowed_paths: vec![],
        },
        load: LoadConfig::Extism {
            file: "plugin.wasm".try_into().unwrap(),
            memory_limit: None,
        },
        api: ApiConfig {
            version: semver::VersionReq::STAR,
            features: vec![],
            host_functions: Some(vec![]),
        },
    }
}

struct Fixture {
    _temp_dir: TempDir,
    location_info: Arc<LocationInfo>,
    plugins: Arc<PluginRegistry>,
    settings: Arc<MemSettings>,
    providers: Arc<MemoryCapabilityRegistry<Arc<dyn ContentProvider>>>,
}

impl Fixture {
    async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            temp_dir.path().join("settings"),
            temp_dir.path().to_path_buf(),
        ));
        tokio::fs::create_dir_all(
            location_info.instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap()),
        )
        .await
        .unwrap();

        Self {
            _temp_dir: temp_dir,
            location_info,
            plugins: Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter))),
            settings: Arc::default(),
            providers: Arc::default(),
        }
    }

    /// The plugin is known with this manifest; whether it is loaded does not matter to the
    /// policy.
    fn install_plugin(&self, allowed_hosts: &[&str]) {
        self.plugins.insert(
            PLUGIN_ID.into(),
            Plugin {
                manifest: manifest(allowed_hosts),
                capabilities: None,
                hash: "hash".into(),
                verified: false,
                state: PluginState::NotLoaded,
            },
        );
    }

    async fn register_plugin_provider(&self, url: String) {
        let provider = PluginProvider {
            metadata: ContentProviderCapabilityMetadata {
                base: CapabilityMetadata {
                    id: "curseforge".into(),
                    name: "CurseForge".into(),
                    description: None,
                    icon: None,
                },
                platform: Some("curseforge".into()),
            },
            url,
        };
        self.providers
            .add(PLUGIN_ID.into(), "curseforge".into(), Arc::new(provider))
            .await
            .unwrap();
    }

    async fn execute(
        &self,
        plan: InstallPlan,
        origin: ProviderId,
    ) -> Result<InstanceId, InstanceError> {
        let downloader = self.downloader();
        let instance = InstanceBuilder::new(
            InstanceId::try_from(INSTANCE_ID).unwrap(),
            "Instance".into(),
            "1.21.1".into(),
            ModLoader::Fabric,
        )
        .build();

        InstallPlanExecutor::new(
            self.providers.clone(),
            Arc::new(NoPlatformProviderSelection),
            Arc::new(OneInstanceStorage(instance)),
            Arc::new(NoCreateInstance),
            Arc::new(NoChangeInstanceGame),
            Arc::new(downloader),
            Arc::new(FsContentFileService::new(self.location_info.clone())),
            Arc::new(SqlitePackStorage::in_memory().await),
            self.location_info.clone(),
            Arc::new(NoProgress),
            Arc::new(NoRemoveInstance),
            Arc::new(NoMetadataCache),
            crate::shared::limits::ContentInstallLimits::default(),
        )
        .execute(
            plan,
            PlanOrigin {
                provider_id: origin,
                content_id: "pack".into(),
            },
        )
        .await
    }

    fn downloader(&self) -> ProviderContentDownloader {
        let semaphore = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(4)));
        let progress: Arc<dyn ProgressService> = Arc::new(NoProgress);
        // The application's client, as the desktop builds it, minus the system proxy, which a
        // loopback test server must not go through.
        let app_client = Arc::new(ReqwestClient::for_downloads(
            progress.clone(),
            Arc::new(
                reqwest_middleware::ClientBuilder::new(
                    reqwest::Client::builder().no_proxy().build().unwrap(),
                )
                .build(),
            ),
            semaphore.clone(),
        ));
        ProviderContentDownloader::new(
            app_client,
            self.plugins.clone(),
            self.settings.clone(),
            semaphore,
            progress,
            crate::shared::limits::ContentInstallLimits::default(),
            crate::shared::limits::PluginSandboxLimits::default(),
        )
        .with_test_network(allow_any_address)
    }

    fn installed(&self, relative_path: &str) -> Option<Vec<u8>> {
        std::fs::read(
            self.location_info
                .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
                .join(relative_path),
        )
        .ok()
    }

    fn instance_is_empty(&self) -> bool {
        std::fs::read_dir(
            self.location_info
                .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap()),
        )
        .unwrap()
        .next()
        .is_none()
    }

    fn staging_is_clean(&self) -> bool {
        let temp = self.location_info.temp_dir();
        !temp.exists() || std::fs::read_dir(temp).unwrap().next().is_none()
    }
}

fn direct_plan(url: String, hash: Option<FileHash>) -> InstallPlan {
    InstallPlan {
        target: InstallTarget::existing(InstanceId::try_from(INSTANCE_ID).unwrap()),
        files: vec![InstallFileEntry::new(
            ContentSource::direct(url, hash),
            "mods/jei.jar",
            ContentType::Mod,
        )],
        overrides: Vec::new(),
    }
}

fn serve_body() -> impl Fn(&str) -> Vec<u8> + Send + Sync + 'static {
    |_| ok(BODY)
}

fn assert_download_error(err: &InstanceError, expected: &str) {
    assert!(
        matches!(err, InstanceError::ContentDownloadError(message) if message.contains(expected)),
        "expected a download error mentioning {expected:?}, got {err:?}"
    );
}

// ─── A plugin's URLs ───────────────────────────────────────────

/// A direct URL from a plugin's plan, on a host outside its `allowed_hosts`,
/// is refused before the host is contacted, and nothing reaches the instance.
#[tokio::test]
async fn should_refuse_a_plugin_url_when_it_is_outside_its_allowed_hosts() {
    let fixture = Fixture::new().await;
    fixture.install_plugin(&[ALLOWED]);
    let other = TestServer::start(OTHER, serve_body()).await;

    let err = fixture
        .execute(
            direct_plan(other.url("/jei.jar"), Some(sha1_of(BODY))),
            plugin_origin(),
        )
        .await
        .unwrap_err();

    assert_download_error(&err, "not listed in the plugin's allowed_hosts");
    assert_eq!(other.connections(), 0, "the host must not be contacted");
    assert!(fixture.instance_is_empty());
    assert!(fixture.staging_is_clean());
}

/// The same plan from a built-in provider downloads fine: the refusal above is the plugin's
/// policy, not the server.
#[tokio::test]
async fn should_download_a_builtin_url_through_the_app_client_when_it_has_no_plugin_origin() {
    let fixture = Fixture::new().await;
    let other = TestServer::start(OTHER, serve_body()).await;

    fixture
        .execute(
            direct_plan(other.url("/jei.jar"), Some(sha1_of(BODY))),
            builtin_origin(),
        )
        .await
        .unwrap();

    assert_eq!(fixture.installed("mods/jei.jar").as_deref(), Some(BODY));
}

#[tokio::test]
async fn should_download_a_plugin_url_when_it_is_on_an_allowed_host() {
    let fixture = Fixture::new().await;
    fixture.install_plugin(&[ALLOWED]);
    let allowed = TestServer::start(ALLOWED, serve_body()).await;

    fixture
        .execute(
            direct_plan(allowed.url("/jei.jar"), Some(sha1_of(BODY))),
            plugin_origin(),
        )
        .await
        .unwrap();

    assert_eq!(fixture.installed("mods/jei.jar").as_deref(), Some(BODY));
    assert!(fixture.staging_is_clean());
}

/// The hosts a user added in the plugin's settings count, as they do for its host functions.
#[tokio::test]
async fn should_honour_hosts_when_they_were_added_in_the_plugin_settings() {
    let fixture = Fixture::new().await;
    fixture.install_plugin(&[]);
    fixture
        .settings
        .upsert(
            PLUGIN_ID,
            &PluginSettings {
                allowed_hosts: vec![ALLOWED.into()],
                ..PluginSettings::default()
            },
        )
        .await
        .unwrap();
    let allowed = TestServer::start(ALLOWED, serve_body()).await;

    fixture
        .execute(direct_plan(allowed.url("/jei.jar"), None), plugin_origin())
        .await
        .unwrap();

    assert_eq!(fixture.installed("mods/jei.jar").as_deref(), Some(BODY));
}

/// One downloader keeps a plugin's client for the files of a plan, but never past its policy.
/// The client vets every redirect hop against the hosts it was built for, so a client kept after
/// the user removed a host would still follow a redirect to it.
#[tokio::test]
async fn should_apply_hosts_when_they_changed_between_two_files_of_a_plan() {
    let fixture = Fixture::new().await;
    fixture.install_plugin(&[]);
    let allow = |hosts: &[&str]| {
        let settings = fixture.settings.clone();
        let hosts = hosts.iter().map(|&host| host.to_owned()).collect();
        async move {
            settings
                .upsert(
                    PLUGIN_ID,
                    &PluginSettings {
                        allowed_hosts: hosts,
                        ..PluginSettings::default()
                    },
                )
                .await
                .unwrap();
        }
    };
    let other = TestServer::start(OTHER, serve_body()).await;
    let moved_to = other.url("/moved.jar");
    let allowed = TestServer::start(ALLOWED, move |path: &str| {
        if path.starts_with("/hop") {
            redirect(302, &moved_to)
        } else {
            ok(BODY)
        }
    })
    .await;
    let downloader = fixture.downloader();
    let staging = fixture.location_info.temp_dir();
    tokio::fs::create_dir_all(&staging).await.unwrap();
    let download = |path: &str, name: &str| {
        let (url, destination) = (allowed.url(path), staging.join(name));
        let downloader = &downloader;
        async move {
            downloader
                .download(&plugin_origin(), &url, None, &destination, None)
                .await
        }
    };

    allow(&[ALLOWED, OTHER]).await;
    download("/a.jar", "a.jar").await.unwrap();

    allow(&[ALLOWED]).await;
    let err = download("/hop.jar", "b.jar").await.unwrap_err();

    assert!(
        matches!(&err, InstanceError::ContentDownloadError(_)),
        "{err:?}"
    );
    assert_eq!(other.connections(), 0);
}

/// A plugin the launcher no longer knows has no policy to go by: nothing is downloaded.
#[tokio::test]
async fn should_refuse_a_url_when_it_is_from_an_unknown_plugin() {
    let fixture = Fixture::new().await;
    let allowed = TestServer::start(ALLOWED, serve_body()).await;

    let err = fixture
        .execute(direct_plan(allowed.url("/jei.jar"), None), plugin_origin())
        .await
        .unwrap_err();

    assert_download_error(&err, "No network policy for plugin");
    assert_eq!(allowed.connections(), 0);
}

/// A ref resolved by a plugin is that plugin's URL, whoever's plan carried the ref: a built-in
/// plan does not lend its policy to it.
#[tokio::test]
async fn should_hold_a_ref_to_that_plugins_hosts_when_a_plugin_resolved_it() {
    let fixture = Fixture::new().await;
    fixture.install_plugin(&[ALLOWED]);
    let other = TestServer::start(OTHER, serve_body()).await;
    fixture
        .register_plugin_provider(other.url("/jei.jar"))
        .await;

    let plan = InstallPlan {
        files: vec![InstallFileEntry::new(
            ContentSource::ProviderRef {
                platform: "curseforge".into(),
                content_id: "238222".into(),
                version: None,
            },
            "mods/jei.jar",
            ContentType::Mod,
        )],
        ..direct_plan(String::new(), None)
    };
    let err = fixture.execute(plan, builtin_origin()).await.unwrap_err();

    assert_download_error(&err, "not listed in the plugin's allowed_hosts");
    assert_eq!(other.connections(), 0);
    assert!(fixture.instance_is_empty());
}

// ─── Hashes ────────────────────────────────────────────────────

/// A body that does not match the plan's hash fails the plan, and the
/// instance stays empty — for a plugin's URL and for a built-in one alike.
#[tokio::test]
async fn should_write_nothing_when_the_hash_does_not_match() {
    for origin in [plugin_origin(), builtin_origin()] {
        let fixture = Fixture::new().await;
        fixture.install_plugin(&[ALLOWED]);
        let allowed = TestServer::start(ALLOWED, serve_body()).await;

        let err = fixture
            .execute(
                direct_plan(allowed.url("/jei.jar"), Some(sha1_of(b"other bytes"))),
                origin.clone(),
            )
            .await
            .unwrap_err();

        assert_download_error(&err, "hash");
        assert_eq!(allowed.connections(), 1, "{origin}");
        assert!(fixture.instance_is_empty(), "{origin}");
        assert!(fixture.staging_is_clean(), "{origin}");
    }
}

/// A file that does not download leaves the instance as it was even when others in the same
/// plan did. (The staging directory is not checked: the failure cancels the other download.)
#[tokio::test]
async fn should_install_no_file_of_a_plan_when_one_fails() {
    let fixture = Fixture::new().await;
    fixture.install_plugin(&[ALLOWED]);
    let allowed = TestServer::start(ALLOWED, serve_body()).await;

    let plan = InstallPlan {
        files: vec![
            InstallFileEntry::new(
                ContentSource::direct(allowed.url("/good.jar"), Some(sha1_of(BODY))),
                "mods/good.jar",
                ContentType::Mod,
            ),
            InstallFileEntry::new(
                ContentSource::direct(allowed.url("/bad.jar"), Some(sha1_of(b"other bytes"))),
                "mods/bad.jar",
                ContentType::Mod,
            ),
        ],
        ..direct_plan(String::new(), None)
    };
    let err = fixture.execute(plan, plugin_origin()).await.unwrap_err();

    assert_download_error(&err, "hash");
    assert!(fixture.instance_is_empty());
}
