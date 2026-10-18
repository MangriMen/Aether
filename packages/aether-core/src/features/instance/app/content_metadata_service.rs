//! What an instance's content list shows a file by — its project's name, description, icon, and
//! its version's number — taken from [`ContentMetadataCache`] and [`ContentIconCache`], and
//! refreshed from the providers and the icons' servers behind the list's back.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant, SystemTime},
};

use dashmap::DashMap;
use futures::{
    FutureExt, StreamExt,
    future::{BoxFuture, Shared},
};

use crate::{
    features::{
        events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
        instance::{
            CachedContentIcon, CachedContentMetadata, CachedContentVersionMetadata, ContentFile,
            ContentIconCache, ContentMetadata, ContentMetadataCache, ContentProvider,
            ContentVersionMetadata, InstanceError, InstanceId, ProviderId,
        },
    },
    shared::capability::domain::CapabilityRegistry,
};

/// How long an id its provider did not know is left alone before it is asked about again.
const UNKNOWN_CONTENT_RETRY_AFTER: Duration = Duration::from_mins(30);

/// How long a provider whose lookup failed (offline, an error, its plugin gone) is left alone
/// before it is asked again, so the lists opened meanwhile do not ask it each time.
const LOOKUP_RETRY_AFTER: Duration = Duration::from_mins(5);

/// How long an icon that failed to arrive is left alone before it is fetched again, so the
/// lists opened meanwhile do not ask for it each time.
const ICON_RETRY_AFTER: Duration = Duration::from_mins(10);

/// How many icons one refresh fetches at once.
const ICON_FETCH_CONCURRENCY: usize = 4;

/// How many cached icons one list looks up at once.
const ICON_LOOKUP_CONCURRENCY: usize = 16;

/// The fetch of one icon, shared by every refresh that wants it: whether a new file arrived.
type IconFetch = Shared<BoxFuture<'static, bool>>;

/// What a provider is asked about: its projects, or its versions. A provider may answer one and
/// not the other, and an id it did not know is unknown as the one or the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Lookup {
    Projects,
    Versions,
}

/// Ids of one provider and one lookup, as their key in the maps below.
type LookupKey = (ProviderId, Lookup);

impl Lookup {
    const ALL: [Self; 2] = [Self::Projects, Self::Versions];

    /// What is asked, for the logs.
    fn what(self) -> &'static str {
        match self {
            Self::Projects => "content metadata",
            Self::Versions => "version metadata",
        }
    }

    /// The provider and the provider's id that `file` is described by: the content id of the
    /// project it was installed from, or the id of the version it was installed as — that
    /// recorded for the provider that updates the file. `None` for a file of no provider (added
    /// by hand and not recognised), or of no recorded id (an empty one, as old records have).
    ///
    /// Not `PackFile::installed_by`: that is the plan that installed the file, a modpack's own
    /// id for every file of the pack.
    fn key(self, file: &ContentFile) -> Option<(&ProviderId, &str)> {
        let provider_id = file.update_provider_id.as_ref()?;
        let info = file.update.as_ref()?.get(provider_id)?;
        let id = match self {
            Self::Projects => info.content_id.as_str(),
            Self::Versions => info.version.as_str(),
        };
        (!id.is_empty()).then_some((provider_id, id))
    }

    /// What `cache` has of `ids` of `provider_id`, fresh or stale.
    async fn read(
        self,
        cache: &dyn ContentMetadataCache,
        provider_id: &ProviderId,
        ids: &[String],
    ) -> Cached {
        match self {
            Self::Projects => Cached::Projects(cache.get_many(provider_id, ids).await),
            Self::Versions => Cached::Versions(cache.get_versions(provider_id, ids).await),
        }
    }

    /// What `provider` answers about `ids`, and the ids its answer leaves out.
    async fn ask(
        self,
        provider: &dyn ContentProvider,
        ids: &[String],
    ) -> Result<(Fetched, Vec<String>), InstanceError> {
        match self {
            Self::Projects => {
                let items = provider.get_contents_metadata(ids).await?;
                let (keyed, unknown) = key_by_asked(ids, items);
                Ok((Fetched::Projects(keyed), unknown))
            }
            Self::Versions => {
                let items = provider.get_versions_metadata(ids).await?;
                let unknown = ids
                    .iter()
                    .filter(|id| !items.iter().any(|item| &item.version_id == *id))
                    .cloned()
                    .collect();
                Ok((Fetched::Versions(items), unknown))
            }
        }
    }
}

/// Whether `id` is among the ids `due` has to ask for under `key`.
fn is_due_id(due: &HashMap<LookupKey, Vec<String>>, key: &LookupKey, id: &str) -> bool {
    due.get(key)
        .is_some_and(|ids| ids.iter().any(|due| due == id))
}

/// What the cache has of one provider's ids of one lookup, by id.
enum Cached {
    Projects(HashMap<String, CachedContentMetadata>),
    Versions(HashMap<String, CachedContentVersionMetadata>),
}

impl Cached {
    /// Whether `id` is missing or stale as of `now`.
    fn is_due(&self, id: &str, now: SystemTime) -> bool {
        match self {
            Self::Projects(items) => items.get(id).is_none_or(|item| item.is_stale(now)),
            Self::Versions(items) => items.get(id).is_none_or(|item| item.is_stale(now)),
        }
    }

    fn project(&self, id: &str) -> Option<ContentMetadata> {
        match self {
            Self::Projects(items) => Some(items.get(id)?.metadata.clone()),
            Self::Versions(_) => None,
        }
    }

    fn version(&self, id: &str) -> Option<ContentVersionMetadata> {
        match self {
            Self::Versions(items) => Some(items.get(id)?.metadata.clone()),
            Self::Projects(_) => None,
        }
    }
}

/// What a provider answered about one lookup, keyed for the cache: projects under the ids they
/// were asked by, versions under their own.
enum Fetched {
    Projects(Vec<(String, ContentMetadata)>),
    Versions(Vec<ContentVersionMetadata>),
}

impl Fetched {
    fn is_empty(&self) -> bool {
        match self {
            Self::Projects(items) => items.is_empty(),
            Self::Versions(items) => items.is_empty(),
        }
    }

    /// Whether it was written.
    async fn store(&self, cache: &dyn ContentMetadataCache, provider_id: &ProviderId) -> bool {
        match self {
            Self::Projects(items) => cache.put_keyed(provider_id, items).await,
            Self::Versions(items) => cache.put_versions(provider_id, items).await,
        }
    }

    /// The icon URLs of what was fetched: of projects; versions have none.
    fn into_icon_urls(self) -> Vec<String> {
        match self {
            Self::Projects(items) => items
                .into_iter()
                .filter_map(|(_, item)| item.icon_url)
                .collect(),
            Self::Versions(_) => Vec::new(),
        }
    }
}

/// Puts the cached [`ContentMetadata`] of their projects, the cached icons of those, and the
/// cached [`ContentVersionMetadata`] of their versions on an instance's content files, and
/// fetches what is missing or stale without making the list wait.
///
/// One per process: it remembers which instances are being refreshed, which providers cannot
/// describe their projects or versions or failed to lately, which ids their provider did not
/// know, and which icons are on their way or failed to arrive.
pub struct ContentMetadataService {
    inner: Arc<Inner>,
}

struct Inner {
    cache: Arc<dyn ContentMetadataCache>,
    icons: Arc<dyn ContentIconCache>,
    providers: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    event_emitter: SharedEventEmitter,
    /// Instances with a refresh running, and whether a list opened meanwhile had work of its
    /// own: it starts no second refresh, and the running one tells the UI to list again once
    /// over, which starts it.
    refreshing: Mutex<HashMap<InstanceId, bool>>,
    /// Providers that answered [`InstanceError::UnsupportedOperation`] to a lookup: not asked
    /// it again until [`ContentMetadataService::forget_unsupported`] clears its plugin.
    unsupported: Mutex<HashSet<LookupKey>>,
    /// Lookups that failed otherwise, with when: not asked again before
    /// [`LOOKUP_RETRY_AFTER`].
    failed: Mutex<HashMap<LookupKey, Instant>>,
    /// Ids their provider left out of its answer, with when: nothing is cached for them, and
    /// they are not asked about again before [`UNKNOWN_CONTENT_RETRY_AFTER`].
    unknown: Mutex<HashMap<(ProviderId, Lookup, String), Instant>>,
    /// Icons being fetched, by URL: a refresh that wants one already on its way waits for that
    /// fetch rather than starting another.
    icon_fetches: Mutex<HashMap<String, IconFetch>>,
    /// Icons that failed to arrive, with when: not fetched again before [`ICON_RETRY_AFTER`].
    icon_failures: Mutex<HashMap<String, Instant>>,
}

impl ContentMetadataService {
    pub fn new(
        cache: Arc<dyn ContentMetadataCache>,
        icons: Arc<dyn ContentIconCache>,
        providers: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        event_emitter: SharedEventEmitter,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                cache,
                icons,
                providers,
                event_emitter,
                refreshing: Mutex::new(HashMap::new()),
                unsupported: Mutex::new(HashSet::new()),
                failed: Mutex::new(HashMap::new()),
                unknown: Mutex::new(HashMap::new()),
                icon_fetches: Mutex::new(HashMap::new()),
                icon_failures: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Ask the providers of `plugin_id` again about the lookups they called unsupported: the
    /// plugin changed, and may answer them now.
    pub fn forget_unsupported(&self, plugin_id: &str) {
        self.inner
            .unsupported
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(provider_id, _)| provider_id.plugin_id != plugin_id);
    }

    /// Set each file's [`ContentFile::metadata`] and [`ContentFile::version_metadata`] to what
    /// the cache has for them, fresh or stale, and its [`ContentFile::icon_path`] to the cached
    /// icon of that metadata, if its file is there. Then start refreshing in the background what
    /// is missing or stale: the metadata of projects and versions first, then the icons — those
    /// of the metadata just fetched too. A file whose version is among what is refreshed is
    /// [`ContentFile::version_metadata_pending`].
    ///
    /// Reads only the caches. Once a refresh of metadata is over, whatever came of it, or one of
    /// icons has brought any, an [`InstanceEventType::ContentSynced`] event of the instance
    /// tells the UI to list it again — the one after the metadata comes before the icons are
    /// fetched; a refresh that fails keeps the stale items and the icons there are.
    pub async fn attach(&self, instance_id: &InstanceId, files: &DashMap<String, ContentFile>) {
        let mut wanted: HashMap<LookupKey, HashSet<String>> = HashMap::new();
        for file in files {
            for lookup in Lookup::ALL {
                if let Some((provider_id, id)) = lookup.key(&file) {
                    wanted
                        .entry((provider_id.clone(), lookup))
                        .or_default()
                        .insert(id.to_owned());
                }
            }
        }
        if wanted.is_empty() {
            return;
        }

        let now = SystemTime::now();
        let mut found: HashMap<LookupKey, Cached> = HashMap::new();
        let mut due: HashMap<LookupKey, Vec<String>> = HashMap::new();
        for ((provider_id, lookup), ids) in wanted {
            let ids: Vec<String> = ids.into_iter().collect();
            let cached = lookup.read(&*self.inner.cache, &provider_id, &ids).await;
            let stale = self
                .inner
                .due_ids(&provider_id, lookup, ids, |id| cached.is_due(id, now));
            if !stale.is_empty() {
                due.insert((provider_id.clone(), lookup), stale);
            }
            found.insert((provider_id, lookup), cached);
        }

        for mut file in files.iter_mut() {
            let metadata = Lookup::Projects.key(&file).and_then(|(provider_id, id)| {
                found
                    .get(&(provider_id.clone(), Lookup::Projects))?
                    .project(id)
            });
            let version_key = Lookup::Versions
                .key(&file)
                .map(|(provider_id, id)| ((provider_id.clone(), Lookup::Versions), id));
            let version_metadata = version_key
                .as_ref()
                .and_then(|(key, id)| found.get(key)?.version(id));
            let version_metadata_pending = version_metadata.is_none()
                && version_key.is_some_and(|(key, id)| is_due_id(&due, &key, id));
            let metadata_pending = metadata
                .as_ref()
                .is_none_or(|metadata| metadata.author.is_none())
                && Lookup::Projects
                    .key(&file)
                    .is_some_and(|(provider_id, id)| {
                        is_due_id(&due, &(provider_id.clone(), Lookup::Projects), id)
                    });
            file.metadata = metadata;
            file.metadata_pending = metadata_pending;
            file.version_metadata = version_metadata;
            file.version_metadata_pending = version_metadata_pending;
        }

        let icon_urls: HashSet<String> = files
            .iter()
            .filter_map(|file| file.metadata.as_ref()?.icon_url.clone())
            .collect();
        let icons = self.inner.lookup_icons(icon_urls).await;
        for mut file in files.iter_mut() {
            let icon_path = file
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.icon_url.as_ref())
                .and_then(|url| match icons.get(url)? {
                    CachedContentIcon::Present {
                        asset_id,
                        revision: Some(revision),
                        ..
                    } => Some(format!("{asset_id}?v={revision}")),
                    CachedContentIcon::Present { asset_id, .. } => Some(asset_id.clone()),
                    CachedContentIcon::Missing => None,
                });
            file.icon_path = icon_path;
        }
        let due_icons = self.inner.due_icons(icons);

        if !due.is_empty() || !due_icons.is_empty() {
            self.spawn_refresh(instance_id, due, due_icons);
        }
    }

    fn spawn_refresh(
        &self,
        instance_id: &InstanceId,
        due: HashMap<LookupKey, Vec<String>>,
        mut due_icons: HashSet<String>,
    ) {
        let Some(guard) = RefreshGuard::acquire(&self.inner, instance_id) else {
            return;
        };

        tokio::spawn(async move {
            // Over whatever came of it: the list it brings shows no file as pending any more,
            // as each asked id is now cached, unknown, or of a provider that failed lately.
            // Told before the icons are looked for and fetched, which may take longer.
            let metadata_asked = !due.is_empty();
            let icon_urls = guard.inner.refresh(due).await;
            if metadata_asked {
                guard.inner.emit_synced(&guard.instance_id).await;
            }
            let icons = guard.inner.lookup_icons(icon_urls).await;
            due_icons.extend(guard.inner.due_icons(icons));
            let icons_arrived = guard.inner.fetch_icons(due_icons).await;

            // Released before the event: the list it brings may need a refresh of its own.
            let inner = guard.inner.clone();
            let instance_id = guard.instance_id.clone();
            let asked_again = guard.release();

            if icons_arrived || asked_again {
                inner.emit_synced(&instance_id).await;
            }
        });
    }
}

/// An instance marked as being refreshed, until released or dropped — also when the refresh
/// panics.
struct RefreshGuard {
    inner: Arc<Inner>,
    instance_id: InstanceId,
    released: bool,
}

impl RefreshGuard {
    /// `None` when a refresh of the instance is running: that one is then marked as asked
    /// again.
    fn acquire(inner: &Arc<Inner>, instance_id: &InstanceId) -> Option<Self> {
        let mut refreshing = inner
            .refreshing
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(asked_again) = refreshing.get_mut(instance_id) {
            *asked_again = true;
            return None;
        }
        refreshing.insert(instance_id.clone(), false);

        Some(Self {
            inner: inner.clone(),
            instance_id: instance_id.clone(),
            released: false,
        })
    }

    /// Unmark the instance. Whether a list asked for a refresh of its own meanwhile.
    fn release(mut self) -> bool {
        self.released = true;
        self.unmark()
    }

    fn unmark(&self) -> bool {
        self.inner
            .refreshing
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.instance_id)
            .unwrap_or_default()
    }
}

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        // Once released, the mark may already be that of the next refresh.
        if !self.released {
            self.unmark();
        }
    }
}

impl Inner {
    async fn emit_synced(&self, instance_id: &InstanceId) {
        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::ContentSynced,
                instance_id: instance_id.clone(),
            })
            .await;
    }

    fn is_unsupported(&self, provider_id: &ProviderId, lookup: Lookup) -> bool {
        self.unsupported
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(&(provider_id.clone(), lookup))
    }

    fn is_known_unknown(&self, provider_id: &ProviderId, lookup: Lookup, id: &str) -> bool {
        self.unknown
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(provider_id.clone(), lookup, id.to_owned()))
            .is_some_and(|at| at.elapsed() < UNKNOWN_CONTENT_RETRY_AFTER)
    }

    fn remember_unknown(&self, provider_id: &ProviderId, lookup: Lookup, ids: Vec<String>) {
        let now = Instant::now();
        let mut unknown = self.unknown.lock().unwrap_or_else(PoisonError::into_inner);
        unknown.retain(|_, at| at.elapsed() < UNKNOWN_CONTENT_RETRY_AFTER);
        unknown.extend(
            ids.into_iter()
                .map(|id| ((provider_id.clone(), lookup, id), now)),
        );
    }

    fn has_failed_lately(&self, provider_id: &ProviderId, lookup: Lookup) -> bool {
        self.failed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(provider_id.clone(), lookup))
            .is_some_and(|at| at.elapsed() < LOOKUP_RETRY_AFTER)
    }

    fn remember_failure(&self, provider_id: &ProviderId, lookup: Lookup) {
        let mut failed = self.failed.lock().unwrap_or_else(PoisonError::into_inner);
        failed.retain(|_, at| at.elapsed() < LOOKUP_RETRY_AFTER);
        failed.insert((provider_id.clone(), lookup), Instant::now());
    }

    /// Of `ids`, those to ask `provider_id` about: missing or stale by `is_due`, and not left
    /// out of its answer lately. None when it cannot answer the lookup, or failed to lately.
    fn due_ids(
        &self,
        provider_id: &ProviderId,
        lookup: Lookup,
        ids: Vec<String>,
        is_due: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        if self.is_unsupported(provider_id, lookup) || self.has_failed_lately(provider_id, lookup) {
            return Vec::new();
        }
        ids.into_iter()
            .filter(|id| is_due(id))
            .filter(|id| !self.is_known_unknown(provider_id, lookup, id))
            .collect()
    }

    /// The registered provider of `provider_id`; `None` when its plugin is off or gone, and the
    /// file is shown by what it has.
    async fn provider(&self, provider_id: &ProviderId) -> Option<Arc<dyn ContentProvider>> {
        let entry = self
            .providers
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await;
        if entry.is_err() {
            tracing::debug!("No content provider {provider_id} to describe its content");
        }
        entry.ok().map(|entry| entry.capability)
    }

    /// Note a failed lookup: a provider that cannot answer it is not asked it again; one that
    /// failed otherwise is not asked it again before [`LOOKUP_RETRY_AFTER`].
    fn lookup_failed(&self, provider_id: &ProviderId, lookup: Lookup, err: InstanceError) {
        let what = lookup.what();
        match err {
            InstanceError::UnsupportedOperation { .. } => {
                tracing::debug!("Content provider {provider_id} does not give {what}");
                self.unsupported
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert((provider_id.clone(), lookup));
            }
            err => {
                tracing::warn!("Could not refresh {what} from {provider_id}: {err}");
                self.remember_failure(provider_id, lookup);
            }
        }
    }

    /// Ask each provider about its due ids, all providers and lookups at once. The icon URLs
    /// of what was cached.
    async fn refresh(&self, due: HashMap<LookupKey, Vec<String>>) -> HashSet<String> {
        futures::future::join_all(
            due.iter()
                .map(|((provider_id, lookup), ids)| self.refresh_of(provider_id, *lookup, ids)),
        )
        .await
        .into_iter()
        .flatten()
        .flat_map(Fetched::into_icon_urls)
        .collect()
    }

    /// What was cached of `ids` by `lookup`; `None` when nothing was.
    async fn refresh_of(
        &self,
        provider_id: &ProviderId,
        lookup: Lookup,
        ids: &[String],
    ) -> Option<Fetched> {
        let Some(provider) = self.provider(provider_id).await else {
            self.remember_failure(provider_id, lookup);
            return None;
        };

        match lookup.ask(&*provider, ids).await {
            Ok((fetched, unknown)) => {
                self.remember_unknown(provider_id, lookup, unknown);
                if fetched.is_empty() {
                    return None;
                }
                if !fetched.store(&*self.cache, provider_id).await {
                    // Still due once listed again: held back, or each list would fetch it anew.
                    tracing::warn!("Could not cache {} from {provider_id}", lookup.what());
                    self.remember_failure(provider_id, lookup);
                }
                Some(fetched)
            }
            Err(err) => {
                self.lookup_failed(provider_id, lookup, err);
                None
            }
        }
    }

    /// What the icon cache has of `urls`, [`ICON_LOOKUP_CONCURRENCY`] at a time.
    async fn lookup_icons(&self, urls: HashSet<String>) -> HashMap<String, CachedContentIcon> {
        futures::stream::iter(urls)
            .map(|url| async move {
                let icon = self.icons.lookup(&url).await;
                (url, icon)
            })
            .buffer_unordered(ICON_LOOKUP_CONCURRENCY)
            .collect()
            .await
    }

    /// Of `icons`, the URLs to fetch: missing or due to be revalidated, and not failed lately.
    fn due_icons(&self, icons: HashMap<String, CachedContentIcon>) -> HashSet<String> {
        let failures = self
            .icon_failures
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        icons
            .into_iter()
            .filter(|(_, icon)| match icon {
                CachedContentIcon::Missing => true,
                CachedContentIcon::Present { revalidate, .. } => *revalidate,
            })
            .map(|(url, _)| url)
            .filter(|url| {
                failures
                    .get(url)
                    .is_none_or(|at| at.elapsed() >= ICON_RETRY_AFTER)
            })
            .collect()
    }

    /// Fetch the icons of `urls`, [`ICON_FETCH_CONCURRENCY`] at a time. Whether any new one
    /// arrived.
    async fn fetch_icons(self: &Arc<Self>, urls: HashSet<String>) -> bool {
        futures::stream::iter(urls)
            .map(|url| self.fetch_icon(url))
            .buffer_unordered(ICON_FETCH_CONCURRENCY)
            .fold(false, |arrived, new| async move { arrived || new })
            .await
    }

    /// The fetch of the icon of `url`: the one on its way, or a new one. A failure is logged
    /// and remembered, and is no new icon.
    fn fetch_icon(self: &Arc<Self>, url: String) -> IconFetch {
        let mut fetches = self
            .icon_fetches
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(fetch) = fetches.get(&url) {
            return fetch.clone();
        }

        let inner = self.clone();
        let key = url.clone();
        // Not polled before it is in the map, so its removal at the end cannot come first.
        let fetch = async move {
            let arrived = match inner.icons.fetch(&url).await {
                Ok(arrived) => arrived,
                Err(err) => {
                    tracing::debug!("Could not fetch the content icon {url}: {err}");
                    inner.remember_icon_failure(&url);
                    false
                }
            };
            inner
                .icon_fetches
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&url);
            arrived
        }
        .boxed()
        .shared();
        fetches.insert(key, fetch.clone());
        fetch
    }

    fn remember_icon_failure(&self, url: &str) {
        let mut failures = self
            .icon_failures
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        failures.retain(|_, at| at.elapsed() < ICON_RETRY_AFTER);
        failures.insert(url.to_owned(), Instant::now());
    }
}

/// The provider's `items` keyed for the cache, and the asked ids none of them answers.
///
/// Every item goes under its own id; one that answers an asked id by its slug (the provider
/// takes either) goes under the asked id as well, or that id would stay missing for good.
fn key_by_asked(
    asked: &[String],
    items: Vec<ContentMetadata>,
) -> (Vec<(String, ContentMetadata)>, Vec<String>) {
    let mut keyed = Vec::with_capacity(items.len());
    let mut unknown = Vec::new();

    for content_id in asked {
        if items.iter().any(|item| &item.content_id == content_id) {
            continue;
        }
        match items
            .iter()
            .find(|item| item.slug.eq_ignore_ascii_case(content_id))
        {
            Some(item) => keyed.push((content_id.clone(), item.clone())),
            None => unknown.push(content_id.clone()),
        }
    }
    keyed.extend(
        items
            .into_iter()
            .map(|item| (item.content_id.clone(), item)),
    );

    (keyed, unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(content_id: &str, slug: &str) -> ContentMetadata {
        ContentMetadata {
            content_id: content_id.into(),
            slug: slug.into(),
            name: slug.into(),
            description: None,
            author: None,
            icon_url: None,
        }
    }

    #[test]
    fn should_key_an_item_also_by_the_asked_id_when_asked_by_slug() {
        let asked = vec![
            "AANobbMI".to_owned(),
            "Lithium".to_owned(),
            "gone".to_owned(),
        ];
        let items = vec![
            metadata("AANobbMI", "sodium"),
            metadata("gvQqBUqZ", "lithium"),
        ];

        let (keyed, unknown) = key_by_asked(&asked, items);

        let keys: Vec<&str> = keyed.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["Lithium", "AANobbMI", "gvQqBUqZ"]);
        assert_eq!(keyed[0].1.content_id, "gvQqBUqZ");
        assert_eq!(unknown, ["gone"]);
    }
}
