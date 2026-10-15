//! `ListContentUseCase` putting cached project and version metadata on the content files, and
//! `ContentMetadataService` refreshing what is missing or stale — metadata and icons — behind
//! the list's back.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use dashmap::DashMap;
use tempfile::TempDir;
use tokio::sync::{Semaphore, mpsc};

use crate::{
    features::{
        events::{Event, EventEmitter, EventError, InstanceEvent, InstanceEventType},
        instance::{
            CONTENT_METADATA_TTL, CONTENT_VERSION_METADATA_TTL, CachedContentIcon,
            CachedContentMetadata, CachedContentVersionMetadata, CapabilityMetadata,
            ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentFile,
            ContentFileUpdateInfo, ContentIconCache, ContentIconError, ContentItem,
            ContentMetadata, ContentMetadataCache, ContentMetadataService, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentVersion, ContentVersionMetadata, Instance, InstanceError, ListContentUseCase,
            PackFile, PackStorage, ProviderId, infra::SqlitePackStorage,
        },
        settings::LocationInfo,
    },
    shared::capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
};

const INSTANCE_ID: &str = "instance";

fn provider_id(capability_id: &str) -> ProviderId {
    ProviderId {
        plugin_id: "plugin".into(),
        capability_id: capability_id.into(),
    }
}

/// The version id the fixture records for a file of `content_id`.
fn version_id(content_id: &str) -> String {
    format!("{content_id}-v1")
}

fn version(content_id: &str, version_number: &str) -> ContentVersionMetadata {
    ContentVersionMetadata {
        version_id: version_id(content_id),
        version_number: version_number.into(),
        name: None,
    }
}

fn metadata(content_id: &str, name: &str) -> ContentMetadata {
    ContentMetadata {
        content_id: content_id.into(),
        slug: name.to_lowercase(),
        name: name.into(),
        description: None,
        author: None,
        icon_url: None,
    }
}

// ── Fakes ──

#[derive(Default)]
struct MemoryMetadataCache {
    items: Mutex<HashMap<(ProviderId, String), CachedContentMetadata>>,
    versions: Mutex<HashMap<(ProviderId, String), CachedContentVersionMetadata>>,
}

impl MemoryMetadataCache {
    fn seed_version(
        &self,
        provider_id: &ProviderId,
        item: ContentVersionMetadata,
        fetched_at: SystemTime,
    ) {
        self.versions.lock().unwrap().insert(
            (provider_id.clone(), item.version_id.clone()),
            CachedContentVersionMetadata {
                metadata: item,
                fetched_at,
            },
        );
    }

    fn seed(&self, provider_id: &ProviderId, item: ContentMetadata, fetched_at: SystemTime) {
        self.items.lock().unwrap().insert(
            (provider_id.clone(), item.content_id.clone()),
            CachedContentMetadata {
                metadata: item,
                fetched_at,
            },
        );
    }
}

#[async_trait]
impl ContentMetadataCache for MemoryMetadataCache {
    async fn get_many(
        &self,
        provider_id: &ProviderId,
        content_ids: &[String],
    ) -> HashMap<String, CachedContentMetadata> {
        let items = self.items.lock().unwrap();
        content_ids
            .iter()
            .filter_map(|id| {
                let item = items.get(&(provider_id.clone(), id.clone()))?;
                Some((id.clone(), item.clone()))
            })
            .collect()
    }

    async fn put_keyed(&self, provider_id: &ProviderId, items: &[(String, ContentMetadata)]) {
        let mut cached = self.items.lock().unwrap();
        for (content_id, item) in items {
            cached.insert(
                (provider_id.clone(), content_id.clone()),
                CachedContentMetadata {
                    metadata: item.clone(),
                    fetched_at: SystemTime::now(),
                },
            );
        }
    }

    async fn get_versions(
        &self,
        provider_id: &ProviderId,
        version_ids: &[String],
    ) -> HashMap<String, CachedContentVersionMetadata> {
        let versions = self.versions.lock().unwrap();
        version_ids
            .iter()
            .filter_map(|id| {
                let item = versions.get(&(provider_id.clone(), id.clone()))?;
                Some((id.clone(), item.clone()))
            })
            .collect()
    }

    async fn put_versions(&self, provider_id: &ProviderId, items: &[ContentVersionMetadata]) {
        let mut cached = self.versions.lock().unwrap();
        for item in items {
            cached.insert(
                (provider_id.clone(), item.version_id.clone()),
                CachedContentVersionMetadata {
                    metadata: item.clone(),
                    fetched_at: SystemTime::now(),
                },
            );
        }
    }
}

/// Icons by URL, as asset ids. A fetch of a missing icon brings it; of a present one, finds it
/// unchanged.
#[derive(Default)]
struct FakeIcons {
    /// URL to asset id and whether it is due to be revalidated.
    present: Mutex<HashMap<String, (String, bool)>>,
    failing: Mutex<HashSet<String>>,
    /// Each fetch's URL, in the order asked.
    fetches: Mutex<Vec<String>>,
    /// When set, a fetch waits for a permit before answering.
    gate: Option<Arc<Semaphore>>,
}

impl FakeIcons {
    fn gated(gate: Arc<Semaphore>) -> Self {
        Self {
            gate: Some(gate),
            ..Self::default()
        }
    }

    fn seed(&self, url: &str, revalidate: bool) {
        self.present
            .lock()
            .unwrap()
            .insert(url.into(), (asset_id(url), revalidate));
    }

    fn fail(&self, url: &str) {
        self.failing.lock().unwrap().insert(url.into());
    }

    /// The URLs fetched, sorted.
    fn fetches(&self) -> Vec<String> {
        let mut fetches = self.fetches.lock().unwrap().clone();
        fetches.sort();
        fetches
    }
}

fn asset_id(url: &str) -> String {
    format!("content-icons/{}", url.rsplit('/').next().unwrap())
}

#[async_trait]
impl ContentIconCache for FakeIcons {
    async fn lookup(&self, url: &str) -> CachedContentIcon {
        match self.present.lock().unwrap().get(url) {
            Some((asset_id, revalidate)) => CachedContentIcon::Present {
                asset_id: asset_id.clone(),
                revalidate: *revalidate,
            },
            None => CachedContentIcon::Missing,
        }
    }

    async fn fetch(&self, url: &str) -> Result<bool, ContentIconError> {
        self.fetches.lock().unwrap().push(url.into());
        if let Some(gate) = &self.gate {
            gate.acquire().await.unwrap().forget();
        }
        if self.failing.lock().unwrap().contains(url) {
            return Err(ContentIconError::Request("offline".into()));
        }

        let mut present = self.present.lock().unwrap();
        if let Some((_, revalidate)) = present.get_mut(url) {
            *revalidate = false;
            return Ok(false);
        }
        present.insert(url.into(), (asset_id(url), false));
        Ok(true)
    }

    async fn remove_unused(&self, _: Duration) -> usize {
        0
    }
}

enum Answer<T> {
    Items(Vec<T>),
    Fail,
    Unsupported,
}

impl<T: Clone> Answer<T> {
    fn answer(
        &self,
        provider: &FakeProvider,
        operation: &'static str,
    ) -> Result<Vec<T>, InstanceError> {
        match self {
            Answer::Items(items) => Ok(items.clone()),
            Answer::Fail => Err(InstanceError::ContentDownloadError("offline".into())),
            Answer::Unsupported => Err(provider.unsupported(operation)),
        }
    }
}

struct FakeProvider {
    metadata: ContentProviderCapabilityMetadata,
    answer: Answer<ContentMetadata>,
    /// What it answers about versions: by default, that it cannot.
    versions: Answer<ContentVersionMetadata>,
    /// Each call's ids, sorted.
    calls: Mutex<Vec<Vec<String>>>,
    /// Each call's version ids, sorted.
    version_calls: Mutex<Vec<Vec<String>>>,
    /// When set, a call waits for a permit before answering.
    gate: Option<Arc<Semaphore>>,
}

impl FakeProvider {
    fn new(capability_id: &str, answer: Answer<ContentMetadata>) -> Self {
        Self {
            metadata: ContentProviderCapabilityMetadata {
                base: CapabilityMetadata {
                    id: capability_id.into(),
                    name: capability_id.into(),
                    description: None,
                    icon: None,
                },
                platform: None,
            },
            answer,
            versions: Answer::Unsupported,
            calls: Mutex::new(Vec::new()),
            version_calls: Mutex::new(Vec::new()),
            gate: None,
        }
    }

    fn with_versions(mut self, versions: Answer<ContentVersionMetadata>) -> Self {
        self.versions = versions;
        self
    }

    fn version_calls(&self) -> Vec<Vec<String>> {
        self.version_calls.lock().unwrap().clone()
    }

    fn gated(mut self, gate: Arc<Semaphore>) -> Self {
        self.gate = Some(gate);
        self
    }

    fn calls(&self) -> Vec<Vec<String>> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl ContentProvider for FakeProvider {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.metadata
    }

    fn provider_id(&self) -> ProviderId {
        provider_id(&self.metadata.base.id)
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

    async fn check_compatibility(
        &self,
        _: &[Instance],
        _: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        unimplemented!()
    }

    async fn get_contents_metadata(
        &self,
        content_ids: &[String],
    ) -> Result<Vec<ContentMetadata>, InstanceError> {
        let mut asked = content_ids.to_vec();
        asked.sort();
        self.calls.lock().unwrap().push(asked);

        if let Some(gate) = &self.gate {
            gate.acquire().await.unwrap().forget();
        }

        self.answer.answer(self, "get_contents_metadata")
    }

    async fn get_versions_metadata(
        &self,
        version_ids: &[String],
    ) -> Result<Vec<ContentVersionMetadata>, InstanceError> {
        let mut asked = version_ids.to_vec();
        asked.sort();
        self.version_calls.lock().unwrap().push(asked);

        self.versions.answer(self, "get_versions_metadata")
    }
}

struct ChannelEvents(mpsc::UnboundedSender<Event>);

#[async_trait]
impl EventEmitter<Event> for ChannelEvents {
    async fn emit(&self, event: Event) -> Result<(), EventError> {
        let _ = self.0.send(event);
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

// ── Fixture ──

struct Fixture {
    _temp: TempDir,
    mods: std::path::PathBuf,
    use_case: ListContentUseCase,
    service: Arc<ContentMetadataService>,
    cache: Arc<MemoryMetadataCache>,
    icons: Arc<FakeIcons>,
    events: mpsc::UnboundedReceiver<Event>,
}

impl Fixture {
    /// An instance with one mod per `(file, provider, content id)`, the providers registered.
    async fn new(files: &[(&str, &str, &str)], providers: Vec<Arc<FakeProvider>>) -> Self {
        Self::with_icons(files, providers, FakeIcons::default()).await
    }

    async fn with_icons(
        files: &[(&str, &str, &str)],
        providers: Vec<Arc<FakeProvider>>,
        icons: FakeIcons,
    ) -> Self {
        let temp = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        ));
        let mods = location_info.instance_dir(INSTANCE_ID).join("mods");
        tokio::fs::create_dir_all(&mods).await.unwrap();

        let pack_storage = Arc::new(SqlitePackStorage::in_memory().await);
        for (file, capability_id, content_id) in files {
            tokio::fs::write(mods.join(file), b"jar").await.unwrap();
            let provider = provider_id(capability_id);
            let mut pack_file = PackFile::from_hash((*file).to_owned(), "hash".into());
            pack_file.update_provider_id = Some(provider.clone());
            pack_file.update = Some(HashMap::from([(
                provider,
                ContentFileUpdateInfo {
                    content_id: (*content_id).to_owned(),
                    version: version_id(content_id),
                },
            )]));
            pack_storage
                .update_pack_file(INSTANCE_ID, &format!("mods/{file}"), &pack_file)
                .await
                .unwrap();
        }

        let registry = Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default());
        for provider in providers {
            let id = provider.provider_id();
            registry
                .add(id.plugin_id, id.capability_id, provider)
                .await
                .unwrap();
        }

        let cache = Arc::new(MemoryMetadataCache::default());
        let icons = Arc::new(icons);
        let (sender, events) = mpsc::unbounded_channel();
        let service = Arc::new(ContentMetadataService::new(
            cache.clone(),
            icons.clone(),
            registry,
            Arc::new(ChannelEvents(sender)),
        ));

        Self {
            _temp: temp,
            mods,
            use_case: ListContentUseCase::new(pack_storage, location_info, service.clone()),
            service,
            cache,
            icons,
            events,
        }
    }

    async fn list(&self) -> DashMap<String, ContentFile> {
        self.use_case.execute(INSTANCE_ID.to_owned()).await.unwrap()
    }

    /// The name of the metadata on `file`, if any.
    async fn names(&self) -> HashMap<String, Option<String>> {
        self.list()
            .await
            .into_iter()
            .map(|(path, file)| (path, file.metadata.map(|m| m.name)))
            .collect()
    }

    /// The version number of the version metadata on each file, if any.
    async fn version_numbers(&self) -> HashMap<String, Option<String>> {
        self.list()
            .await
            .into_iter()
            .map(|(path, file)| (path, file.version_metadata.map(|v| v.version_number)))
            .collect()
    }

    /// The icon path on each file.
    async fn icon_paths(&self) -> HashMap<String, Option<String>> {
        self.list()
            .await
            .into_iter()
            .map(|(path, file)| (path, file.icon_path))
            .collect()
    }

    async fn next_event(&mut self) -> InstanceEvent {
        let event = tokio::time::timeout(Duration::from_secs(5), self.events.recv())
            .await
            .expect("no event in time")
            .expect("the emitter is gone");
        event.try_into().expect("not an instance event")
    }

    /// Let a refresh started by a list run to its end. The fakes never wait on anything but the
    /// gate, so a few turns of the (current-thread) runtime are enough.
    async fn settle() {
        for _ in 0..20 {
            tokio::task::yield_now().await;
        }
    }

    fn assert_no_event(&mut self) {
        assert!(self.events.try_recv().is_err(), "unexpected event");
    }
}

// ── Tests ──

#[tokio::test]
async fn should_list_without_waiting_for_the_provider_when_metadata_is_missing() {
    let gate = Arc::new(Semaphore::new(0));
    let provider = Arc::new(
        FakeProvider::new("modrinth", Answer::Items(vec![metadata("B", "Lithium")]))
            .gated(gate.clone()),
    );
    let fixture = Fixture::new(
        &[("a.jar", "modrinth", "A"), ("b.jar", "modrinth", "B")],
        vec![provider.clone()],
    )
    .await;
    fixture.cache.seed(
        &provider_id("modrinth"),
        metadata("A", "Sodium"),
        SystemTime::now(),
    );

    let names = tokio::time::timeout(Duration::from_secs(5), fixture.names())
        .await
        .expect("the list waited for the provider");

    assert_eq!(names["mods/a.jar"], Some("Sodium".into()));
    assert_eq!(names["mods/b.jar"], None);
    Fixture::settle().await;
    assert_eq!(provider.calls(), [["B"]]);
    gate.add_permits(1);
}

#[tokio::test]
async fn should_not_ask_the_provider_when_everything_is_cached_and_fresh() {
    let provider = Arc::new(
        FakeProvider::new("modrinth", Answer::Items(vec![])).with_versions(Answer::Items(vec![])),
    );
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider.clone()]).await;
    let now = SystemTime::now();
    let modrinth = provider_id("modrinth");
    fixture.cache.seed(&modrinth, metadata("A", "Sodium"), now);
    // Fresh long after a project would be stale: a version does not change.
    fixture.cache.seed_version(
        &modrinth,
        version("A", "0.6.13"),
        now - CONTENT_METADATA_TTL * 2,
    );

    assert_eq!(fixture.names().await["mods/a.jar"], Some("Sodium".into()));
    assert_eq!(
        fixture.version_numbers().await["mods/a.jar"],
        Some("0.6.13".into())
    );
    Fixture::settle().await;

    assert!(provider.calls().is_empty());
    assert!(provider.version_calls().is_empty());
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_ask_each_provider_once_then_cache_and_tell_the_ui_when_metadata_is_missing() {
    let modrinth = Arc::new(FakeProvider::new(
        "modrinth",
        Answer::Items(vec![metadata("A", "Sodium"), metadata("B", "Lithium")]),
    ));
    let curse = Arc::new(FakeProvider::new(
        "curse",
        Answer::Items(vec![metadata("C", "JEI")]),
    ));
    let mut fixture = Fixture::new(
        &[
            ("a.jar", "modrinth", "A"),
            ("b.jar", "modrinth", "B"),
            ("c.jar", "curse", "C"),
        ],
        vec![modrinth.clone(), curse.clone()],
    )
    .await;

    let first = fixture.names().await;
    let event = fixture.next_event().await;

    assert!(first.values().all(Option::is_none));
    assert_eq!(event.instance_id, INSTANCE_ID);
    assert!(matches!(event.event, InstanceEventType::Synced));
    assert_eq!(modrinth.calls(), [["A", "B"]]);
    assert_eq!(curse.calls(), [["C"]]);

    let second = fixture.names().await;
    assert_eq!(second["mods/a.jar"], Some("Sodium".into()));
    assert_eq!(second["mods/b.jar"], Some("Lithium".into()));
    assert_eq!(second["mods/c.jar"], Some("JEI".into()));
    Fixture::settle().await;
    assert_eq!(modrinth.calls().len(), 1);
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_keep_the_stale_item_when_the_refresh_fails() {
    let provider = Arc::new(FakeProvider::new("modrinth", Answer::Fail));
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider.clone()]).await;
    let long_ago = SystemTime::now() - CONTENT_METADATA_TTL - Duration::from_hours(1);
    fixture
        .cache
        .seed(&provider_id("modrinth"), metadata("A", "Sodium"), long_ago);

    assert_eq!(fixture.names().await["mods/a.jar"], Some("Sodium".into()));
    Fixture::settle().await;
    assert_eq!(provider.calls().len(), 1);
    fixture.assert_no_event();

    // Still shown, and asked again: a failure is not remembered.
    assert_eq!(fixture.names().await["mods/a.jar"], Some("Sodium".into()));
    Fixture::settle().await;
    assert_eq!(provider.calls().len(), 2);
}

#[tokio::test]
async fn should_not_ask_again_a_provider_that_cannot_describe_its_content() {
    let provider = Arc::new(FakeProvider::new("modrinth", Answer::Unsupported));
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider.clone()]).await;

    assert_eq!(fixture.names().await["mods/a.jar"], None);
    Fixture::settle().await;
    assert_eq!(fixture.names().await["mods/a.jar"], None);
    Fixture::settle().await;

    assert_eq!(provider.calls().len(), 1);
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_start_one_refresh_when_lists_overlap() {
    let gate = Arc::new(Semaphore::new(0));
    let provider = Arc::new(
        FakeProvider::new("modrinth", Answer::Items(vec![metadata("A", "Sodium")]))
            .gated(gate.clone()),
    );
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider.clone()]).await;

    fixture.list().await;
    Fixture::settle().await;
    fixture.list().await;
    Fixture::settle().await;
    assert_eq!(provider.calls().len(), 1);

    gate.add_permits(1);
    fixture.next_event().await;
    assert_eq!(fixture.names().await["mods/a.jar"], Some("Sodium".into()));
    Fixture::settle().await;
    assert_eq!(provider.calls().len(), 1);
}

#[tokio::test]
async fn should_find_an_item_answered_by_slug_and_not_ask_again_about_unknown_ids() {
    let provider = Arc::new(FakeProvider::new(
        "modrinth",
        Answer::Items(vec![metadata("AANobbMI", "Sodium")]),
    ));
    let mut fixture = Fixture::new(
        &[
            ("a.jar", "modrinth", "sodium"),
            ("b.jar", "modrinth", "gone"),
        ],
        vec![provider.clone()],
    )
    .await;

    fixture.list().await;
    fixture.next_event().await;
    let names = fixture.names().await;
    Fixture::settle().await;

    assert_eq!(names["mods/a.jar"], Some("Sodium".into()));
    assert_eq!(names["mods/b.jar"], None);
    assert_eq!(provider.calls(), [["gone", "sodium"]]);
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_leave_a_file_of_no_provider_without_metadata() {
    let provider = Arc::new(FakeProvider::new("modrinth", Answer::Items(vec![])));
    let fixture = Fixture::new(&[], vec![provider.clone()]).await;
    tokio::fs::write(fixture.mods.join("manual.jar"), b"jar")
        .await
        .unwrap();

    let names = fixture.names().await;
    Fixture::settle().await;

    assert_eq!(names.get("mods/manual.jar"), Some(&None));
    assert!(provider.calls().is_empty());
}

// ── Icons ──

const ICON_X: &str = "https://cdn.example.com/x.webp";
const ICON_Y: &str = "https://cdn.example.com/y.png";

fn with_icon(mut item: ContentMetadata, url: &str) -> ContentMetadata {
    item.icon_url = Some(url.into());
    item
}

#[tokio::test]
async fn should_fetch_an_icon_shared_by_many_files_once_and_tell_the_ui_when_it_arrives() {
    let provider = Arc::new(FakeProvider::new("modrinth", Answer::Items(vec![])));
    let mut fixture = Fixture::new(
        &[
            ("a.jar", "modrinth", "A"),
            ("b.jar", "modrinth", "B"),
            ("c.jar", "modrinth", "C"),
        ],
        vec![provider.clone()],
    )
    .await;
    let modrinth = provider_id("modrinth");
    let now = SystemTime::now();
    fixture
        .cache
        .seed(&modrinth, with_icon(metadata("A", "Sodium"), ICON_X), now);
    fixture
        .cache
        .seed(&modrinth, with_icon(metadata("B", "Lithium"), ICON_X), now);
    fixture
        .cache
        .seed(&modrinth, with_icon(metadata("C", "JEI"), ICON_Y), now);

    let before = fixture.icon_paths().await;
    let event = fixture.next_event().await;

    assert!(before.values().all(Option::is_none), "{before:?}");
    assert!(matches!(event.event, InstanceEventType::Synced));
    assert_eq!(fixture.icons.fetches(), [ICON_X, ICON_Y]);

    let after = fixture.icon_paths().await;
    assert_eq!(after["mods/a.jar"], Some(asset_id(ICON_X)));
    assert_eq!(after["mods/b.jar"], Some(asset_id(ICON_X)));
    assert_eq!(after["mods/c.jar"], Some(asset_id(ICON_Y)));
    Fixture::settle().await;
    assert_eq!(fixture.icons.fetches().len(), 2);
    assert!(provider.calls().is_empty());
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_fetch_the_icons_of_metadata_just_fetched_before_telling_the_ui_once() {
    let provider = Arc::new(FakeProvider::new(
        "modrinth",
        Answer::Items(vec![with_icon(metadata("A", "Sodium"), ICON_X)]),
    ));
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider.clone()]).await;

    fixture.list().await;
    fixture.next_event().await;

    assert_eq!(fixture.icons.fetches(), [ICON_X]);
    let files = fixture.list().await;
    let file = files.get("mods/a.jar").unwrap();
    assert_eq!(file.metadata.as_ref().unwrap().name, "Sodium");
    assert_eq!(file.icon_path, Some(asset_id(ICON_X)));
    drop(file);
    Fixture::settle().await;
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_wait_for_an_icon_already_on_its_way_rather_than_fetch_it_again() {
    let gate = Arc::new(Semaphore::new(0));
    let provider = Arc::new(FakeProvider::new("modrinth", Answer::Items(vec![])));
    let mut fixture = Fixture::with_icons(
        &[("a.jar", "modrinth", "A")],
        vec![provider],
        FakeIcons::gated(gate.clone()),
    )
    .await;
    fixture.cache.seed(
        &provider_id("modrinth"),
        with_icon(metadata("A", "Sodium"), ICON_X),
        SystemTime::now(),
    );

    // The same files listed for another instance while the first fetch waits.
    let files = fixture.list().await;
    Fixture::settle().await;
    fixture.service.attach("other", &files).await;
    Fixture::settle().await;
    assert_eq!(fixture.icons.fetches(), [ICON_X]);

    gate.add_permits(1);
    let mut told: Vec<String> = vec![
        fixture.next_event().await.instance_id,
        fixture.next_event().await.instance_id,
    ];
    told.sort();

    assert_eq!(told, [INSTANCE_ID, "other"]);
    assert_eq!(fixture.icons.fetches(), [ICON_X]);
}

#[tokio::test]
async fn should_not_fetch_again_soon_an_icon_that_failed() {
    let provider = Arc::new(FakeProvider::new("modrinth", Answer::Items(vec![])));
    let mut fixture = Fixture::new(
        &[("a.jar", "modrinth", "A"), ("b.jar", "modrinth", "B")],
        vec![provider],
    )
    .await;
    let modrinth = provider_id("modrinth");
    let now = SystemTime::now();
    fixture
        .cache
        .seed(&modrinth, with_icon(metadata("A", "Sodium"), ICON_X), now);
    fixture
        .cache
        .seed(&modrinth, with_icon(metadata("B", "Lithium"), ICON_Y), now);
    fixture.icons.fail(ICON_X);

    fixture.list().await;
    // Y arrived, X did not: an event for what did.
    fixture.next_event().await;
    let paths = fixture.icon_paths().await;
    Fixture::settle().await;

    assert_eq!(paths["mods/a.jar"], None);
    assert_eq!(paths["mods/b.jar"], Some(asset_id(ICON_Y)));
    assert_eq!(fixture.icons.fetches(), [ICON_X, ICON_Y]);
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_show_a_cached_icon_while_revalidating_it_and_stay_quiet_when_unchanged() {
    let provider = Arc::new(FakeProvider::new("modrinth", Answer::Items(vec![])));
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider]).await;
    fixture.cache.seed(
        &provider_id("modrinth"),
        with_icon(metadata("A", "Sodium"), ICON_Y),
        SystemTime::now(),
    );
    fixture.icons.seed(ICON_Y, true);

    let paths = fixture.icon_paths().await;
    Fixture::settle().await;

    assert_eq!(paths["mods/a.jar"], Some(asset_id(ICON_Y)));
    assert_eq!(fixture.icons.fetches(), [ICON_Y]);
    fixture.assert_no_event();

    fixture.icon_paths().await;
    Fixture::settle().await;
    assert_eq!(fixture.icons.fetches().len(), 1);
}

// ── Versions ──

#[tokio::test]
async fn should_ask_for_versions_once_with_the_projects_then_cache_and_tell_the_ui_once() {
    let modrinth = Arc::new(
        FakeProvider::new(
            "modrinth",
            Answer::Items(vec![metadata("A", "Sodium"), metadata("B", "Lithium")]),
        )
        .with_versions(Answer::Items(vec![
            version("A", "0.6.13+mc1.21.4"),
            version("B", "mc1.21.4-0.14.7"),
        ])),
    );
    let mut fixture = Fixture::new(
        &[("a.jar", "modrinth", "A"), ("b.jar", "modrinth", "B")],
        vec![modrinth.clone()],
    )
    .await;

    let first = fixture.version_numbers().await;
    fixture.next_event().await;
    Fixture::settle().await;
    // Projects and versions refreshed together: one event for both.
    fixture.assert_no_event();

    assert!(first.values().all(Option::is_none));
    assert_eq!(modrinth.calls(), [["A", "B"]]);
    assert_eq!(
        modrinth.version_calls(),
        [[version_id("A"), version_id("B")]]
    );

    let files = fixture.list().await;
    let file = files.get("mods/a.jar").unwrap();
    assert_eq!(file.metadata.as_ref().unwrap().name, "Sodium");
    assert_eq!(file.version_metadata, Some(version("A", "0.6.13+mc1.21.4")));
    assert_eq!(
        files.get("mods/b.jar").unwrap().version_metadata,
        Some(version("B", "mc1.21.4-0.14.7"))
    );
    drop(file);
    Fixture::settle().await;
    assert_eq!(modrinth.version_calls().len(), 1);
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_ask_only_for_the_missing_versions_when_the_projects_are_fresh() {
    let provider = Arc::new(
        FakeProvider::new("modrinth", Answer::Items(vec![]))
            .with_versions(Answer::Items(vec![version("B", "2.0")])),
    );
    let mut fixture = Fixture::new(
        &[("a.jar", "modrinth", "A"), ("b.jar", "modrinth", "B")],
        vec![provider.clone()],
    )
    .await;
    let modrinth = provider_id("modrinth");
    let now = SystemTime::now();
    fixture.cache.seed(&modrinth, metadata("A", "Sodium"), now);
    fixture.cache.seed(&modrinth, metadata("B", "Lithium"), now);
    fixture
        .cache
        .seed_version(&modrinth, version("A", "1.0"), now);

    fixture.list().await;
    fixture.next_event().await;
    let numbers = fixture.version_numbers().await;

    assert!(provider.calls().is_empty());
    assert_eq!(provider.version_calls(), [[version_id("B")]]);
    assert_eq!(numbers["mods/a.jar"], Some("1.0".into()));
    assert_eq!(numbers["mods/b.jar"], Some("2.0".into()));
}

#[tokio::test]
async fn should_keep_the_stale_version_when_its_refresh_fails() {
    let provider =
        Arc::new(FakeProvider::new("modrinth", Answer::Items(vec![])).with_versions(Answer::Fail));
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider.clone()]).await;
    let modrinth = provider_id("modrinth");
    fixture
        .cache
        .seed(&modrinth, metadata("A", "Sodium"), SystemTime::now());
    let long_ago = SystemTime::now() - CONTENT_VERSION_METADATA_TTL - Duration::from_hours(1);
    fixture
        .cache
        .seed_version(&modrinth, version("A", "1.0"), long_ago);

    assert_eq!(
        fixture.version_numbers().await["mods/a.jar"],
        Some("1.0".into())
    );
    Fixture::settle().await;
    assert_eq!(provider.version_calls().len(), 1);
    fixture.assert_no_event();

    // Still shown, and asked again: a failure is not remembered.
    assert_eq!(
        fixture.version_numbers().await["mods/a.jar"],
        Some("1.0".into())
    );
    Fixture::settle().await;
    assert_eq!(provider.version_calls().len(), 2);
}

#[tokio::test]
async fn should_not_ask_again_for_versions_a_provider_cannot_describe() {
    let provider = Arc::new(FakeProvider::new(
        "modrinth",
        Answer::Items(vec![metadata("A", "Sodium")]),
    ));
    let mut fixture = Fixture::new(&[("a.jar", "modrinth", "A")], vec![provider.clone()]).await;

    fixture.list().await;
    // The projects still arrive.
    fixture.next_event().await;
    let names = fixture.names().await;
    Fixture::settle().await;
    let numbers = fixture.version_numbers().await;
    Fixture::settle().await;

    assert_eq!(names["mods/a.jar"], Some("Sodium".into()));
    assert_eq!(numbers["mods/a.jar"], None);
    assert_eq!(provider.version_calls().len(), 1);
    fixture.assert_no_event();
}

#[tokio::test]
async fn should_not_ask_again_soon_about_versions_the_provider_did_not_know() {
    let provider = Arc::new(
        FakeProvider::new("modrinth", Answer::Items(vec![]))
            .with_versions(Answer::Items(vec![version("A", "1.0")])),
    );
    let mut fixture = Fixture::new(
        &[("a.jar", "modrinth", "A"), ("b.jar", "modrinth", "B")],
        vec![provider.clone()],
    )
    .await;
    let modrinth = provider_id("modrinth");
    let now = SystemTime::now();
    fixture.cache.seed(&modrinth, metadata("A", "Sodium"), now);
    fixture.cache.seed(&modrinth, metadata("B", "Lithium"), now);

    fixture.list().await;
    fixture.next_event().await;
    let numbers = fixture.version_numbers().await;
    Fixture::settle().await;

    assert_eq!(numbers["mods/a.jar"], Some("1.0".into()));
    assert_eq!(numbers["mods/b.jar"], None);
    assert_eq!(
        provider.version_calls(),
        [[version_id("A"), version_id("B")]]
    );
    fixture.assert_no_event();
}
