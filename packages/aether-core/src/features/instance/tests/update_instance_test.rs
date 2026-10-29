//! `UpdateInstanceUseCase` updating a pack through the real executor: the version it asks for
//! and records, the stage through a change of game, the files of a pack installed before owners
//! were recorded, and the plugin updater for a provider without plans.

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::{
        events::{EventError, ProgressBarId, ProgressEventType, ProgressService},
        instance::{
            CachedContentAuthor, CachedContentMetadata, CachedContentVersionMetadata,
            CapabilityMetadata, ChangeInstanceGameUseCase, ContentAuthor,
            ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentDownloader,
            ContentItem, ContentMetadata, ContentMetadataCache, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentSource, ContentType, ContentVersion, ContentVersionMetadata,
            CreateInstanceUseCasePort, InstallFileEntry, InstallPlan, InstallPlanExecutor,
            InstallTarget, Instance, InstanceBuilder, InstanceError, InstanceGame, InstanceId,
            InstanceInstallService, InstanceInstallStage, InstanceLocks, InstanceStorage,
            NewInstance, PackFile, PackInfo, PackMetadata, PackSource, PackStorage, PlanOrigin,
            ProviderId, RemoveInstanceUseCasePort, ResolveInstallPlanParams, ResolveTarget,
            UpdateInstanceUseCase, UpdateInstanceUseCasePort, Updater, UpdaterCapabilityMetadata,
            infra::{FsContentFileService, NoPlatformProviderSelection, SqlitePackStorage},
        },
        minecraft::{
            LoaderVersionPreference, LoaderVersionService, MinecraftDomainError, ModLoader, modded,
        },
        settings::LocationInfo,
    },
    shared::{
        capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
        hash::{FileHash, HashAlgorithm},
        request_client::BytesProgress,
    },
};

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

const INSTANCE_ID: &str = "instance";
const PACK_ID: &str = "pack";
const PLUGIN: &str = "plugin";
const CAPABILITY: &str = "provider";

fn provider_id() -> ProviderId {
    ProviderId::from_parts(PLUGIN, CAPABILITY).unwrap()
}

fn origin() -> PlanOrigin {
    PlanOrigin {
        provider_id: provider_id(),
        content_id: PACK_ID.into(),
    }
}

fn url(name: &str) -> String {
    format!("https://cdn.example/{name}")
}

fn sha1(body: &str) -> String {
    sha1_smol::Sha1::from(body).hexdigest()
}

/// A file of the pack, hashed as the downloader below writes it: its URL is its body.
fn pack_file(name: &str, path: &str) -> InstallFileEntry {
    InstallFileEntry::new(
        ContentSource::direct(
            url(name),
            Some(FileHash::new(HashAlgorithm::Sha1, sha1(&url(name))).unwrap()),
        ),
        path,
        ContentType::Mod,
    )
}

fn neoforge_1_21_4() -> InstanceGame {
    InstanceGame {
        game_version: "1.21.4".into(),
        mod_loader: ModLoader::NeoForge,
        loader_version: Some(LoaderVersionPreference::Exact("21.4.1".into())),
    }
}

// ─── Mocks ─────────────────────────────────────────────────────

/// A version of the pack: the game it names, and its files.
struct PackVersion {
    game: Option<InstanceGame>,
    files: Vec<InstallFileEntry>,
}

/// The provider of the pack. Its newest version is `v2`; it plans only the versions it has.
struct PackProvider {
    metadata: ContentProviderCapabilityMetadata,
    versions: HashMap<String, PackVersion>,
    /// `false` — a plugin that deploys its packs itself.
    plans: bool,
    /// `false` — no `resolve_pack_metadata`.
    describes: bool,
    asked_metadata: Mutex<Vec<PackSource>>,
    asked_plans: Mutex<Vec<ResolveInstallPlanParams>>,
}

impl PackProvider {
    fn new(versions: impl IntoIterator<Item = (&'static str, PackVersion)>) -> Self {
        Self {
            metadata: ContentProviderCapabilityMetadata {
                base: CapabilityMetadata {
                    id: CAPABILITY.into(),
                    name: CAPABILITY.into(),
                    description: None,
                    icon: None,
                },
                platform: None,
            },
            versions: versions
                .into_iter()
                .map(|(id, version)| (id.to_owned(), version))
                .collect(),
            plans: true,
            describes: true,
            asked_metadata: Mutex::default(),
            asked_plans: Mutex::default(),
        }
    }

    /// `v1` has `a.jar` and `b.jar`; `v2` replaces `a.jar` with `a2.jar`.
    fn v1_and_v2(v2_game: Option<InstanceGame>) -> Self {
        Self::new([
            (
                "v1",
                PackVersion {
                    game: None,
                    files: vec![
                        pack_file("a.jar", "mods/a.jar"),
                        pack_file("b.jar", "mods/b.jar"),
                    ],
                },
            ),
            (
                "v2",
                PackVersion {
                    game: v2_game,
                    files: vec![
                        pack_file("a2.jar", "mods/a2.jar"),
                        pack_file("b.jar", "mods/b.jar"),
                    ],
                },
            ),
        ])
    }

    fn asked_plans(&self) -> Vec<ResolveInstallPlanParams> {
        self.asked_plans.lock().unwrap().clone()
    }
}

#[async_trait]
impl ContentProvider for PackProvider {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.metadata
    }

    fn provider_id(&self) -> ProviderId {
        provider_id()
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

    async fn resolve_install_plan(
        &self,
        params: ResolveInstallPlanParams,
    ) -> Result<InstallPlan, InstanceError> {
        if !self.plans {
            return Err(self.unsupported("resolve_install_plan"));
        }
        self.asked_plans.lock().unwrap().push(params.clone());

        let version_id = params.version.unwrap_or_else(|| "v2".into());
        let version =
            self.versions
                .get(&version_id)
                .ok_or_else(|| InstanceError::ContentProviderError {
                    reason: format!("no version {version_id}"),
                })?;
        let ResolveTarget::Existing { instance_id, .. } = params.target else {
            unimplemented!("updates only")
        };

        Ok(InstallPlan {
            target: InstallTarget::Existing {
                instance_id,
                game: version.game.clone(),
            },
            files: version.files.clone(),
            overrides: Vec::new(),
        })
    }

    async fn resolve_pack_metadata(
        &self,
        source: PackSource,
    ) -> Result<PackMetadata, InstanceError> {
        if !self.describes {
            return Err(self.unsupported("resolve_pack_metadata"));
        }
        self.asked_metadata.lock().unwrap().push(source);

        Ok(PackMetadata {
            name: "Pack".into(),
            version_id: Some("v2".into()),
            version_name: None,
            description: None,
            game_version: "1.21.1".into(),
            mod_loader: ModLoader::Fabric,
            loader_version: None,
            file_count: None,
            icon_url: None,
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

/// Writes the URL as the file's body; fails for a URL containing `fail`.
struct UrlDownloader;

#[async_trait]
impl ContentDownloader for UrlDownloader {
    async fn download(
        &self,
        _: &ProviderId,
        url: &str,
        _: Option<&FileHash>,
        destination: &Path,
        _: Option<BytesProgress<'_>>,
    ) -> Result<u64, InstanceError> {
        if url.contains("fail") {
            return Err(InstanceError::ContentDownloadError(url.into()));
        }
        tokio::fs::write(destination, url).await.unwrap();
        Ok(url.len() as u64)
    }
}

#[derive(Default)]
struct RecordingUpdater {
    updated: Mutex<Vec<String>>,
}

#[async_trait]
impl Updater for RecordingUpdater {
    fn metadata(&self) -> &UpdaterCapabilityMetadata {
        unimplemented!()
    }

    async fn update(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        self.updated.lock().unwrap().push(instance_id.to_string());
        Ok(())
    }
}

#[derive(Default)]
struct MemInstanceStorage {
    instances: Mutex<HashMap<String, Instance>>,
}

impl MemInstanceStorage {
    fn instance(&self) -> Instance {
        self.instances.lock().unwrap()[INSTANCE_ID].clone()
    }
}

#[async_trait]
impl InstanceStorage for MemInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(self.instances.lock().unwrap().values().cloned().collect())
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        self.instances
            .lock()
            .unwrap()
            .get(id.as_str())
            .cloned()
            .ok_or_else(|| InstanceError::NotFound {
                instance_id: id.to_string(),
            })
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        self.instances
            .lock()
            .unwrap()
            .insert(instance.id().to_string(), instance.clone());
        Ok(())
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

/// Installs a game as the real install use case moves the stage: an instance whose pack is
/// being deployed stays `PackInstalling`, any other becomes `Installed`. Records the stage it
/// found.
struct StageKeepingInstall {
    storage: Arc<MemInstanceStorage>,
    stages: Mutex<Vec<InstanceInstallStage>>,
}

#[async_trait]
impl InstanceInstallService for StageKeepingInstall {
    async fn execute(
        &self,
        instance_id: InstanceId,
        _: bool,
        _: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        let mut instance = self.storage.get(&instance_id).await?;
        self.stages.lock().unwrap().push(instance.install_stage);
        if instance.install_stage != InstanceInstallStage::PackInstalling {
            instance.install_stage = InstanceInstallStage::Installed;
        }
        self.storage.upsert(&instance).await
    }
}

struct AnyLoaderVersion;

#[async_trait]
impl LoaderVersionService for AnyLoaderVersion {
    async fn resolve(
        &self,
        _: &str,
        _: &ModLoader,
        _: Option<&LoaderVersionPreference>,
    ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
        Ok(None)
    }

    async fn try_get_default(
        &self,
        _: &str,
        _: &ModLoader,
    ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError> {
        Ok(None)
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
        unimplemented!("updates create no instance")
    }

    async fn execute_for_pack(
        &self,
        _: NewInstance,
        _: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        unimplemented!("updates create no instance")
    }
}

struct NoRemoveInstance;

#[async_trait]
impl RemoveInstanceUseCasePort for NoRemoveInstance {
    async fn execute(&self, _: InstanceId) -> Result<(), InstanceError> {
        unimplemented!("updates remove no instance")
    }
}

struct NoProgress;

#[async_trait]
impl ProgressService for NoProgress {
    async fn init_progress(
        &self,
        _: ProgressEventType,
        _: f64,
        _: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn init_or_edit_progress(
        &self,
        _: Option<ProgressBarId>,
        _: ProgressEventType,
        _: f64,
        _: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn emit_progress(
        &self,
        _: &ProgressBarId,
        _: f64,
        _: Option<&str>,
    ) -> Result<(), EventError> {
        Ok(())
    }
    async fn edit_progress(
        &self,
        _: &ProgressBarId,
        _: ProgressEventType,
        _: f64,
        _: String,
    ) -> Result<(), EventError> {
        Ok(())
    }
}

// ─── Fixture ───────────────────────────────────────────────────

struct Fixture {
    _temp_dir: TempDir,
    location_info: Arc<LocationInfo>,
    storage: Arc<MemInstanceStorage>,
    pack_storage: Arc<SqlitePackStorage>,
    install: Arc<StageKeepingInstall>,
    updater: Arc<RecordingUpdater>,
    locks: Arc<InstanceLocks>,
    use_case: UpdateInstanceUseCase,
}

impl Fixture {
    /// An installed Fabric 1.21.1 instance of the pack's `v1`, its pack served by `provider`
    /// (`None` — by no content provider, only an updater).
    async fn new(provider: Option<Arc<PackProvider>>) -> Self {
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

        let storage = Arc::new(MemInstanceStorage::default());
        let mut instance = InstanceBuilder::new(
            InstanceId::try_from(INSTANCE_ID).unwrap(),
            "Instance".into(),
            "1.21.1".into(),
            ModLoader::Fabric,
        )
        .with_pack_info(Some(PackInfo {
            provider_id: provider_id(),
            modpack_id: PACK_ID.into(),
            version_id: "v1".into(),
        }))
        .build();
        instance.install_stage = InstanceInstallStage::Installed;
        storage.upsert(&instance).await.unwrap();

        let providers = Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default());
        if let Some(provider) = provider {
            providers
                .add(PLUGIN.into(), CAPABILITY.into(), provider)
                .await
                .unwrap();
        }
        let updater = Arc::new(RecordingUpdater::default());
        let updaters = Arc::new(MemoryCapabilityRegistry::<Arc<dyn Updater>>::default());
        updaters
            .add(PLUGIN.into(), CAPABILITY.into(), updater.clone())
            .await
            .unwrap();

        let install = Arc::new(StageKeepingInstall {
            storage: storage.clone(),
            stages: Mutex::default(),
        });
        let pack_storage = Arc::new(SqlitePackStorage::in_memory().await);
        let executor = Arc::new(InstallPlanExecutor::new(
            providers.clone(),
            Arc::new(NoPlatformProviderSelection),
            storage.clone(),
            Arc::new(NoCreateInstance),
            Arc::new(ChangeInstanceGameUseCase::new(
                storage.clone(),
                Arc::new(AnyLoaderVersion),
                install.clone(),
            )),
            Arc::new(UrlDownloader),
            Arc::new(FsContentFileService::new(location_info.clone())),
            pack_storage.clone(),
            location_info.clone(),
            Arc::new(NoProgress),
            Arc::new(NoRemoveInstance),
            Arc::new(NoMetadataCache),
            crate::shared::limits::ContentInstallLimits::default(),
        ));

        let locks = Arc::new(InstanceLocks::new());
        let use_case = UpdateInstanceUseCase::new(
            storage.clone(),
            updaters,
            providers,
            pack_storage.clone(),
            executor,
            locks.clone(),
        );

        Self {
            _temp_dir: temp_dir,
            location_info,
            storage,
            pack_storage,
            install,
            updater,
            locks,
            use_case,
        }
    }

    /// Put the file `name` of the pack at `path`, recorded as installed by `installed_by`.
    async fn put(&self, name: &str, path: &str, installed_by: Option<PlanOrigin>) {
        let full = self
            .location_info
            .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, url(name)).unwrap();

        self.pack_storage
            .update_pack_file(
                &InstanceId::try_from(INSTANCE_ID).unwrap(),
                path,
                &PackFile {
                    installed_by,
                    ..PackFile::from_hash(name.into(), sha1(&url(name)))
                },
            )
            .await
            .unwrap();
    }

    /// `v1` as the executor installs it: every file recorded as the pack's.
    async fn put_v1(&self, installed_by: Option<PlanOrigin>) {
        self.put("a.jar", "mods/a.jar", installed_by.clone()).await;
        self.put("b.jar", "mods/b.jar", installed_by).await;
    }

    async fn update(&self) -> Result<(), InstanceError> {
        self.use_case
            .execute(InstanceId::try_from(INSTANCE_ID).unwrap())
            .await
    }

    fn mods(&self) -> Vec<String> {
        let mut mods: Vec<String> = std::fs::read_dir(
            self.location_info
                .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
                .join("mods"),
        )
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
        mods.sort();
        mods
    }

    async fn owner_of(&self, path: &str) -> Option<PlanOrigin> {
        self.pack_storage
            .get_pack_files(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .await
            .unwrap()[path]
            .installed_by
            .clone()
    }

    fn version_id(&self) -> String {
        self.storage.instance().pack_info.unwrap().version_id
    }
}

// ─── Updating through a plan ───────────────────────────────────

/// The core case: the update takes the version a check for updates finds from the installed
/// one, applies its plan, and the instance records that version.
#[tokio::test]
async fn should_update_the_pack_through_a_plan_and_record_its_version_when_a_new_version_exists() {
    let provider = Arc::new(PackProvider::v1_and_v2(None));
    let fixture = Fixture::new(Some(provider.clone())).await;
    fixture.put_v1(Some(origin())).await;

    fixture.update().await.unwrap();

    assert_eq!(
        *provider.asked_metadata.lock().unwrap(),
        [PackSource::Content {
            content_id: PACK_ID.into(),
            version: None,
            installed_version: Some("v1".into()),
        }]
    );
    assert_eq!(
        provider.asked_plans(),
        [ResolveInstallPlanParams {
            content_id: PACK_ID.into(),
            version: Some("v2".into()),
            installed_version: Some("v1".into()),
            target: ResolveTarget::Existing {
                instance_id: InstanceId::try_from(INSTANCE_ID).unwrap(),
                game_version: "1.21.1".into(),
                loader: Some(ModLoader::Fabric),
            },
        }]
    );
    assert_eq!(fixture.mods(), ["a2.jar", "b.jar"]);
    assert_eq!(fixture.version_id(), "v2");
    assert_eq!(
        fixture.storage.instance().install_stage,
        InstanceInstallStage::Installed
    );
    assert!(fixture.updater.updated.lock().unwrap().is_empty());
}

/// An update waits for whatever else is changing the instance's content, and keeps the instance
/// from being launched while it runs.
#[tokio::test]
async fn should_wait_for_the_instance_lock_when_another_plan_holds_it() {
    let provider = Arc::new(PackProvider::v1_and_v2(None));
    let fixture = Fixture::new(Some(provider.clone())).await;
    fixture.put_v1(Some(origin())).await;
    let held = fixture
        .locks
        .lock(&InstanceId::try_from(INSTANCE_ID).unwrap())
        .await;

    let mut update = std::pin::pin!(fixture.update());
    assert!(futures::poll!(&mut update).is_pending());
    assert!(provider.asked_plans().is_empty());
    assert_eq!(
        fixture.storage.instance().install_stage,
        InstanceInstallStage::Installed
    );

    drop(held);
    update.await.unwrap();
    assert_eq!(fixture.version_id(), "v2");
    assert!(
        fixture
            .locks
            .try_lock(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .is_ok()
    );
}

/// A version on another game moves the instance to it; installing the game keeps the instance
/// unlaunchable, and the update makes it `Installed` once its files are in.
#[tokio::test]
async fn should_end_installed_when_the_instance_moved_to_the_game_of_the_new_version() {
    let provider = Arc::new(PackProvider::v1_and_v2(Some(neoforge_1_21_4())));
    let fixture = Fixture::new(Some(provider)).await;
    fixture.put_v1(Some(origin())).await;

    fixture.update().await.unwrap();

    let instance = fixture.storage.instance();
    assert_eq!(instance.game_version, "1.21.4");
    assert_eq!(instance.loader, ModLoader::NeoForge);
    assert_eq!(
        *fixture.install.stages.lock().unwrap(),
        [InstanceInstallStage::PackInstalling]
    );
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
    assert_eq!(fixture.version_id(), "v2");
}

/// A failed update leaves the instance in the stage it was in, on the version it records.
#[tokio::test]
async fn should_restore_the_stage_when_the_update_fails() {
    let mut provider = PackProvider::v1_and_v2(None);
    provider.versions.get_mut("v2").unwrap().files = vec![pack_file("fail.jar", "mods/f.jar")];
    let fixture = Fixture::new(Some(Arc::new(provider))).await;
    fixture.put_v1(Some(origin())).await;

    fixture.update().await.unwrap_err();

    assert_eq!(
        fixture.storage.instance().install_stage,
        InstanceInstallStage::Installed
    );
    assert_eq!(fixture.version_id(), "v1");
    assert_eq!(fixture.mods(), ["a.jar", "b.jar"]);
}

/// The network drops on one file of a version on another game: the instance keeps its game,
/// loader, stage, version, files and their records — nothing of the new version gets in.
#[tokio::test]
async fn should_keep_the_instance_as_it_was_when_a_file_of_a_game_change_fails() {
    let mut provider = PackProvider::v1_and_v2(Some(neoforge_1_21_4()));
    provider
        .versions
        .get_mut("v2")
        .unwrap()
        .files
        .push(pack_file("fail.jar", "mods/fail.jar"));
    let fixture = Fixture::new(Some(Arc::new(provider))).await;
    fixture.put_v1(Some(origin())).await;
    let records = || async {
        let records = fixture
            .pack_storage
            .get_pack_files(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .await
            .unwrap();
        format!(
            "{:?}",
            records
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>()
        )
    };
    let records_before = records().await;

    fixture.update().await.unwrap_err();

    let instance = fixture.storage.instance();
    assert_eq!(instance.game_version, "1.21.1");
    assert_eq!(instance.loader, ModLoader::Fabric);
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
    assert!(fixture.install.stages.lock().unwrap().is_empty());
    assert_eq!(fixture.version_id(), "v1");
    assert_eq!(fixture.mods(), ["a.jar", "b.jar"]);
    assert_eq!(records().await, records_before);
}

/// Without `resolve_pack_metadata` the plan picks the version itself; the instance cannot tell
/// which, and keeps the one it records.
#[tokio::test]
async fn should_keep_the_recorded_version_when_the_provider_does_not_name_it() {
    let mut provider = PackProvider::v1_and_v2(None);
    provider.describes = false;
    let provider = Arc::new(provider);
    let fixture = Fixture::new(Some(provider.clone())).await;
    fixture.put_v1(Some(origin())).await;

    fixture.update().await.unwrap();

    assert_eq!(provider.asked_plans()[0].version, None);
    assert_eq!(fixture.mods(), ["a2.jar", "b.jar"]);
    assert_eq!(fixture.version_id(), "v1");
}

// ─── A pack installed before owners were recorded ──────────────

/// The files of the installed version become the pack's before the update, so the one it drops
/// is removed; a mod the user added next to them is not the pack's and stays.
#[tokio::test]
async fn should_adopt_the_files_when_a_pack_was_installed_without_owners() {
    let provider = Arc::new(PackProvider::v1_and_v2(None));
    let fixture = Fixture::new(Some(provider.clone())).await;
    fixture.put_v1(None).await;
    fixture.put("user.jar", "mods/user.jar", None).await;

    fixture.update().await.unwrap();

    assert_eq!(fixture.mods(), ["a2.jar", "b.jar", "user.jar"]);
    assert_eq!(fixture.owner_of("mods/b.jar").await, Some(origin()));
    assert_eq!(fixture.owner_of("mods/user.jar").await, None);
    assert_eq!(
        provider
            .asked_plans()
            .iter()
            .map(|params| params.version.clone())
            .collect::<Vec<_>>(),
        [Some("v2".into()), Some("v1".into())]
    );
}

/// A failed update takes its adoption back with it: the files stay without an owner, as the
/// user may still have to tell them apart.
#[tokio::test]
async fn should_leave_the_files_ownerless_when_the_update_that_adopted_them_fails() {
    let mut provider = PackProvider::v1_and_v2(None);
    provider
        .versions
        .get_mut("v2")
        .unwrap()
        .files
        .push(pack_file("fail.jar", "mods/fail.jar"));
    let fixture = Fixture::new(Some(Arc::new(provider))).await;
    fixture.put_v1(None).await;

    fixture.update().await.unwrap_err();

    assert_eq!(fixture.mods(), ["a.jar", "b.jar"]);
    assert_eq!(fixture.owner_of("mods/a.jar").await, None);
    assert_eq!(fixture.owner_of("mods/b.jar").await, None);
    assert_eq!(fixture.version_id(), "v1");
}

/// The installed version is gone: nothing tells the pack's files from the user's, so none is
/// removed, and the update still goes through.
#[tokio::test]
async fn should_remove_no_ownerless_file_when_the_installed_version_is_gone() {
    let mut provider = PackProvider::v1_and_v2(None);
    provider.versions.remove("v1");
    let fixture = Fixture::new(Some(Arc::new(provider))).await;
    fixture.put_v1(None).await;

    fixture.update().await.unwrap();

    assert_eq!(fixture.mods(), ["a.jar", "a2.jar", "b.jar"]);
    assert_eq!(fixture.owner_of("mods/a.jar").await, None);
    assert_eq!(fixture.version_id(), "v2");
}

// ─── A plugin that deploys its packs itself ────────────────────

#[tokio::test]
async fn should_update_through_the_updater_when_the_provider_has_no_plans() {
    let mut provider = PackProvider::v1_and_v2(None);
    provider.plans = false;
    let fixture = Fixture::new(Some(Arc::new(provider))).await;

    fixture.update().await.unwrap();

    assert_eq!(*fixture.updater.updated.lock().unwrap(), [INSTANCE_ID]);
    assert_eq!(
        fixture.storage.instance().install_stage,
        InstanceInstallStage::Installed
    );
    assert_eq!(fixture.version_id(), "v1");
}

#[tokio::test]
async fn should_update_through_the_updater_when_no_provider_serves_the_pack() {
    let fixture = Fixture::new(None).await;

    fixture.update().await.unwrap();

    assert_eq!(*fixture.updater.updated.lock().unwrap(), [INSTANCE_ID]);
}
