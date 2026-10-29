//! `InstallContentUseCase` asks the provider for a plan and hands it to the executor: what it
//! asks for, under which owner the files are recorded, and the stage a new pack's instance ends in.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::{
        instance::{
            AtomicInstallParams, BaseContentParams, CapabilityMetadata,
            ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentFileUpdateInfo,
            ContentIdentity, ContentInstallParams, ContentItem, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentSource, ContentType, ContentVersion, InstallContentUseCase, InstallFileEntry,
            InstallPlan, InstallPlanExecutorPort, InstallTarget, Instance, InstanceBuilder,
            InstanceError, InstanceId, InstanceInstallStage, InstanceLocks, InstanceStorage,
            ModpackInstallParams, NewInstance, PackFile, PackInfo, PackStorage, PlanOrigin,
            PluginId, ProviderId, ResolveInstallPlanParams, ResolveTarget,
            infra::SqlitePackStorage,
        },
        minecraft::ModLoader,
    },
    shared::capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
};

const PLUGIN: &str = "plugin";
const CAPABILITY: &str = "provider";
const INSTANCE_ID: &str = "instance";
const NEW_INSTANCE_ID: &str = "new-instance";

type Answer = Box<dyn Fn(&ResolveTarget) -> InstallTarget + Send + Sync>;

fn provider_id() -> ProviderId {
    ProviderId::from_parts(PLUGIN, CAPABILITY).unwrap()
}

fn pack_info(modpack_id: &str) -> PackInfo {
    PackInfo {
        provider_id: provider_id(),
        modpack_id: modpack_id.into(),
        version_id: "v1".into(),
    }
}

fn new_instance(pack_info: Option<PackInfo>) -> NewInstance {
    NewInstance {
        name: "Pack".into(),
        game_version: "1.21.1".into(),
        mod_loader: ModLoader::Fabric,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info,
    }
}

// ─── Mocks ─────────────────────────────────────────────────────

/// Answers `resolve_install_plan` with a plan for the target `answer` picks — or, without one,
/// leaves it unimplemented.
struct MockProvider {
    metadata: ContentProviderCapabilityMetadata,
    answer: Option<Answer>,
    /// The files of every plan it answers with.
    files: Vec<InstallFileEntry>,
    asked: Mutex<Vec<ResolveInstallPlanParams>>,
}

impl MockProvider {
    fn new(answer: Option<Answer>) -> Arc<Self> {
        Arc::new(Self {
            metadata: ContentProviderCapabilityMetadata {
                base: CapabilityMetadata {
                    id: CAPABILITY.into(),
                    name: CAPABILITY.into(),
                    description: None,
                    icon: None,
                },
                platform: None,
            },
            answer,
            files: Vec::new(),
            asked: Mutex::default(),
        })
    }

    /// A provider serving `platform` whose plans are the one file of `content_id` there.
    fn planning_file_of(platform: &str, content_id: &str) -> Arc<Self> {
        let mut provider = Arc::into_inner(Self::new(Some(answer_as_asked()))).unwrap();
        provider.metadata.platform = Some(platform.into());
        provider.files = vec![InstallFileEntry::new(
            ContentSource::Direct {
                url: "https://cdn.example/sodium.jar".into(),
                hash: None,
                identity: Some(ContentIdentity {
                    platform: platform.into(),
                    content_id: content_id.into(),
                    version: "v2".into(),
                }),
            },
            "mods/sodium.jar",
            ContentType::Mod,
        )];
        Arc::new(provider)
    }
}

#[async_trait]
impl ContentProvider for MockProvider {
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
        let Some(answer) = &self.answer else {
            return Err(self.unsupported("resolve_install_plan"));
        };
        let target = answer(&params.target);
        self.asked.lock().unwrap().push(params);

        Ok(InstallPlan {
            target,
            files: self.files.clone(),
            overrides: Vec::new(),
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

/// Records the origins it was asked to run; "creates" `NEW_INSTANCE_ID` for a new target.
#[derive(Default)]
struct RecordingExecutor {
    calls: Mutex<Vec<PlanOrigin>>,
}

#[async_trait]
impl InstallPlanExecutorPort for RecordingExecutor {
    async fn execute(
        &self,
        plan: InstallPlan,
        origin: PlanOrigin,
    ) -> Result<InstanceId, InstanceError> {
        self.calls.lock().unwrap().push(origin);
        Ok(match plan.target {
            InstallTarget::Existing { instance_id, .. } => instance_id,
            InstallTarget::New(_) => InstanceId::try_from(NEW_INSTANCE_ID).unwrap(),
        })
    }
}

#[derive(Default)]
struct MemInstanceStorage {
    instances: Mutex<Vec<Instance>>,
}

impl MemInstanceStorage {
    fn stage(&self, id: &str) -> InstanceInstallStage {
        self.instances
            .lock()
            .unwrap()
            .iter()
            .find(|instance| instance.id().as_str() == id)
            .unwrap()
            .install_stage
    }
}

#[async_trait]
impl InstanceStorage for MemInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(self.instances.lock().unwrap().clone())
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        self.instances
            .lock()
            .unwrap()
            .iter()
            .find(|instance| instance.id() == id)
            .cloned()
            .ok_or_else(|| InstanceError::NotFound {
                instance_id: id.to_string(),
            })
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let mut instances = self.instances.lock().unwrap();
        instances.retain(|existing| existing.id() != instance.id());
        instances.push(instance.clone());
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

// ─── Fixture ───────────────────────────────────────────────────

struct Fixture {
    _temp_dir: TempDir,
    executor: Arc<RecordingExecutor>,
    storage: Arc<MemInstanceStorage>,
    pack_storage: Arc<SqlitePackStorage>,
    locks: Arc<InstanceLocks>,
    use_case: InstallContentUseCase,
}

impl Fixture {
    /// An installed instance, and the one the executor "creates" for a pack, mid-install.
    async fn new(provider: &Arc<MockProvider>) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let registry = Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default());
        registry
            .add(PLUGIN.into(), CAPABILITY.into(), provider.clone())
            .await
            .unwrap();

        let storage = Arc::new(MemInstanceStorage::default());
        for (id, pack_info, stage) in [
            (INSTANCE_ID, None, InstanceInstallStage::Installed),
            (
                NEW_INSTANCE_ID,
                Some(pack_info("project-id")),
                InstanceInstallStage::PackInstalling,
            ),
        ] {
            let mut instance = InstanceBuilder::new(
                InstanceId::try_from(id).unwrap(),
                id.into(),
                "1.21.1".into(),
                ModLoader::Fabric,
            )
            .with_pack_info(pack_info)
            .build();
            instance.install_stage = stage;
            storage.upsert(&instance).await.unwrap();
        }

        let executor = Arc::new(RecordingExecutor::default());
        let pack_storage = Arc::new(SqlitePackStorage::in_memory().await);
        let locks = Arc::new(InstanceLocks::new());
        let use_case = InstallContentUseCase::new(
            pack_storage.clone(),
            registry,
            storage.clone(),
            executor.clone(),
            locks.clone(),
        );

        Self {
            _temp_dir: temp_dir,
            executor,
            storage,
            pack_storage,
            locks,
            use_case,
        }
    }
}

fn base(content_id: &str) -> BaseContentParams {
    BaseContentParams {
        content_id: content_id.into(),
        content_version: Some("v2".into()),
        provider_id: provider_id(),
    }
}

fn atomic(content_type: ContentType, loader: Option<&str>) -> ContentInstallParams {
    ContentInstallParams::Atomic(AtomicInstallParams {
        base: base("sodium"),
        instance_id: InstanceId::try_from(INSTANCE_ID).unwrap(),
        game_version: "1.21.1".into(),
        loader: loader.map(|l| l.parse().unwrap()),
        content_type,
    })
}

/// A plan for the very instance asked about, or a pack for a new one.
fn answer_as_asked() -> Answer {
    Box::new(|target| match target {
        ResolveTarget::Existing { instance_id, .. } => InstallTarget::existing(instance_id.clone()),
        ResolveTarget::New => InstallTarget::New(new_instance(Some(pack_info("project-id")))),
    })
}

// ─── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn should_install_content_under_the_content_owner_when_an_instance_is_chosen() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;

    fixture
        .use_case
        .execute(atomic(ContentType::Mod, Some("fabric")))
        .await
        .unwrap();

    assert_eq!(
        *provider.asked.lock().unwrap(),
        vec![ResolveInstallPlanParams {
            content_id: "sodium".into(),
            version: Some("v2".into()),
            installed_version: None,
            target: ResolveTarget::Existing {
                instance_id: InstanceId::try_from(INSTANCE_ID).unwrap(),
                game_version: "1.21.1".into(),
                loader: Some(ModLoader::Fabric),
            },
        }]
    );
    assert_eq!(
        *fixture.executor.calls.lock().unwrap(),
        vec![PlanOrigin {
            provider_id: provider_id(),
            content_id: "sodium".into(),
        }]
    );
    assert_eq!(
        fixture.storage.stage(INSTANCE_ID),
        InstanceInstallStage::Installed
    );
}

/// Content installed into an instance waits for any plan already changing it.
#[tokio::test]
async fn should_wait_for_the_instance_lock_when_another_plan_holds_it() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;
    let held = fixture
        .locks
        .lock(&InstanceId::try_from(INSTANCE_ID).unwrap())
        .await;

    let mut install = std::pin::pin!(
        fixture
            .use_case
            .execute(atomic(ContentType::Mod, Some("fabric")))
    );
    assert!(futures::poll!(&mut install).is_pending());
    assert!(provider.asked.lock().unwrap().is_empty());

    drop(held);
    install.await.unwrap();
    assert_eq!(fixture.executor.calls.lock().unwrap().len(), 1);
}

/// A modpack makes an instance of its own: it does not wait for one being changed.
#[tokio::test]
async fn should_not_wait_for_an_instance_lock_when_installing_a_modpack() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;
    let _held = fixture
        .locks
        .lock(&InstanceId::try_from(INSTANCE_ID).unwrap())
        .await;

    fixture
        .use_case
        .execute(ContentInstallParams::Modpack(ModpackInstallParams {
            base: base("project-id"),
        }))
        .await
        .unwrap();

    assert_eq!(fixture.executor.calls.lock().unwrap().len(), 1);
}

/// The provider picks the next version from the installed one (a beta user stays on betas), so
/// it is told the version recorded for the content in the pack file.
#[tokio::test]
async fn should_tell_the_provider_the_installed_version_when_installing_content() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;
    let mut record = PackFile::from_hash("sodium.jar".into(), "hash".into());
    record.update = Some(HashMap::from([(
        provider_id(),
        ContentFileUpdateInfo {
            content_id: "sodium".into(),
            version: "beta-1".into(),
        },
    )]));
    fixture
        .pack_storage
        .update_pack_file(
            &InstanceId::try_from(INSTANCE_ID).unwrap(),
            "mods/sodium.jar",
            &record,
        )
        .await
        .unwrap();

    fixture
        .use_case
        .execute(atomic(ContentType::Mod, Some("fabric")))
        .await
        .unwrap();

    let asked = provider.asked.lock().unwrap().clone();
    assert_eq!(asked[0].installed_version.as_deref(), Some("beta-1"));
}

#[tokio::test]
async fn should_ask_for_a_vanilla_instance_when_it_has_no_loader() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;

    fixture
        .use_case
        .execute(atomic(ContentType::Mod, Some("vanilla")))
        .await
        .unwrap();

    let asked = provider.asked.lock().unwrap().clone();
    let ResolveTarget::Existing { loader, .. } = &asked[0].target else {
        panic!("expected an existing target")
    };
    assert_eq!(*loader, None);
}

#[tokio::test]
async fn should_own_the_new_instance_by_the_recorded_pack_when_a_modpack_is_installed() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;

    // The caller named the pack by a slug; the provider's plan records the project id.
    fixture
        .use_case
        .execute(ContentInstallParams::Modpack(ModpackInstallParams {
            base: base("some-slug"),
        }))
        .await
        .unwrap();

    assert_eq!(provider.asked.lock().unwrap()[0].target, ResolveTarget::New);
    assert_eq!(
        *fixture.executor.calls.lock().unwrap(),
        vec![PlanOrigin {
            provider_id: provider_id(),
            content_id: "project-id".into(),
        }]
    );
    assert_eq!(
        fixture.storage.stage(NEW_INSTANCE_ID),
        InstanceInstallStage::Installed
    );
}

/// A mod asked for by its slug is owned under the id its file names, so that an update asked
/// for by the id replaces it rather than installing next to it.
#[tokio::test]
async fn should_own_a_mod_under_the_id_of_its_file_when_it_is_asked_for_by_slug() {
    let provider = MockProvider::planning_file_of("modrinth", "AANobbMI");
    let fixture = Fixture::new(&provider).await;

    fixture
        .use_case
        .execute(atomic(ContentType::Mod, Some("fabric")))
        .await
        .unwrap();

    assert_eq!(
        *fixture.executor.calls.lock().unwrap(),
        vec![PlanOrigin {
            provider_id: provider_id(),
            content_id: "AANobbMI".into(),
        }]
    );
}

/// A file of another platform names some other content, not the one asked for.
#[tokio::test]
async fn should_keep_the_asked_id_when_the_file_is_of_another_platform() {
    let provider = MockProvider::planning_file_of("modrinth", "AANobbMI");
    let mut provider = Arc::into_inner(provider).unwrap();
    provider.metadata.platform = Some("curseforge".into());
    let provider = Arc::new(provider);
    let fixture = Fixture::new(&provider).await;

    fixture
        .use_case
        .execute(atomic(ContentType::Mod, Some("fabric")))
        .await
        .unwrap();

    assert_eq!(
        fixture.executor.calls.lock().unwrap()[0].content_id,
        "sodium"
    );
}

fn atomic_into(instance_id: &str, content_id: &str) -> ContentInstallParams {
    ContentInstallParams::Atomic(AtomicInstallParams {
        base: base(content_id),
        instance_id: InstanceId::try_from(instance_id).unwrap(),
        game_version: "1.21.1".into(),
        loader: Some(ModLoader::Fabric),
        content_type: ContentType::Mod,
    })
}

/// A pack's instance left unfinished becomes ready once a plan of that pack runs through — not
/// when some other content is installed into it.
#[tokio::test]
async fn should_finish_a_pack_instance_only_when_the_plan_is_of_its_pack() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;

    fixture
        .use_case
        .execute(atomic_into(NEW_INSTANCE_ID, "sodium"))
        .await
        .unwrap();
    assert_eq!(
        fixture.storage.stage(NEW_INSTANCE_ID),
        InstanceInstallStage::PackInstalling
    );

    fixture
        .use_case
        .execute(atomic_into(NEW_INSTANCE_ID, "project-id"))
        .await
        .unwrap();
    assert_eq!(
        fixture.storage.stage(NEW_INSTANCE_ID),
        InstanceInstallStage::Installed
    );
}

#[tokio::test]
async fn should_refuse_a_plan_when_it_is_for_another_instance_than_the_one_asked_about() {
    let provider = MockProvider::new(Some(Box::new(|_| {
        InstallTarget::existing(InstanceId::try_from("other").unwrap())
    })));
    let fixture = Fixture::new(&provider).await;

    fixture
        .use_case
        .execute(atomic(ContentType::Mod, None))
        .await
        .unwrap_err();

    assert!(fixture.executor.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_refuse_a_new_pack_when_it_belongs_to_another_provider() {
    let provider = MockProvider::new(Some(Box::new(|_| {
        let mut pack_info = pack_info("project-id");
        pack_info.provider_id.plugin_id = PluginId::from_static("other-plugin");
        InstallTarget::New(new_instance(Some(pack_info)))
    })));
    let fixture = Fixture::new(&provider).await;

    fixture
        .use_case
        .execute(ContentInstallParams::Modpack(ModpackInstallParams {
            base: base("some-slug"),
        }))
        .await
        .unwrap_err();

    assert!(fixture.executor.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_not_install_a_modpack_as_atomic_content_when_asked_to() {
    let provider = MockProvider::new(Some(answer_as_asked()));
    let fixture = Fixture::new(&provider).await;

    let err = fixture
        .use_case
        .execute(atomic(ContentType::Modpack, None))
        .await
        .unwrap_err();

    assert!(matches!(err, InstanceError::UnsupportedContentType { .. }));
    assert!(provider.asked.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_fail_when_the_provider_has_no_plans() {
    let provider = MockProvider::new(None);
    let fixture = Fixture::new(&provider).await;

    let err = fixture
        .use_case
        .execute(atomic(ContentType::Mod, Some("fabric")))
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        InstanceError::UnsupportedOperation {
            operation: "resolve_install_plan",
            ..
        }
    ));
    assert!(fixture.executor.calls.lock().unwrap().is_empty());
}
