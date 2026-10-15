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
            CachedContentIcon, ContentFile, ContentIconCache, ContentMetadata,
            ContentMetadataCache, ContentProvider, InstanceError, ProviderId,
        },
    },
    shared::capability::domain::CapabilityRegistry,
};

/// How long an id its provider did not know is left alone before it is asked about again.
const UNKNOWN_CONTENT_RETRY_AFTER: Duration = Duration::from_mins(30);

/// How long an icon that failed to arrive is left alone before it is fetched again, so the
/// lists opened meanwhile do not ask for it each time.
const ICON_RETRY_AFTER: Duration = Duration::from_mins(10);

/// How many icons one refresh fetches at once.
const ICON_FETCH_CONCURRENCY: usize = 4;

/// The fetch of one icon, shared by every refresh that wants it: whether a new file arrived.
type IconFetch = Shared<BoxFuture<'static, bool>>;

/// What a provider is asked about: its projects, or its versions. A provider may answer one and
/// not the other, and an id it did not know is unknown as the one or the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Lookup {
    Projects,
    Versions,
}

impl Lookup {
    /// What is asked, for the logs.
    fn what(self) -> &'static str {
        match self {
            Self::Projects => "content metadata",
            Self::Versions => "version metadata",
        }
    }
}

/// The ids to refresh, by provider: of projects and of versions.
#[derive(Default)]
struct Due {
    projects: HashMap<ProviderId, Vec<String>>,
    versions: HashMap<ProviderId, Vec<String>>,
}

impl Due {
    fn is_empty(&self) -> bool {
        self.projects.is_empty() && self.versions.is_empty()
    }
}

/// The ids a provider's files name, of projects and of versions.
#[derive(Default)]
struct Wanted {
    projects: HashSet<String>,
    versions: HashSet<String>,
}

/// Puts the cached [`ContentMetadata`] of their projects, the cached icons of those, and the
/// cached [`ContentVersionMetadata`] of their versions on an instance's content files, and
/// fetches what is missing or stale without making the list wait.
///
/// One per process: it remembers which instances are being refreshed, which providers cannot
/// describe their projects or versions, which ids their provider did not know, and which icons
/// are on their way or failed to arrive.
///
/// [`ContentVersionMetadata`]: crate::features::instance::ContentVersionMetadata
pub struct ContentMetadataService {
    inner: Arc<Inner>,
}

struct Inner {
    cache: Arc<dyn ContentMetadataCache>,
    icons: Arc<dyn ContentIconCache>,
    providers: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    event_emitter: SharedEventEmitter,
    /// Instances with a refresh running: a list opened meanwhile starts no second one.
    refreshing: Mutex<HashSet<String>>,
    /// Providers that answered [`InstanceError::UnsupportedOperation`] to a lookup: not asked
    /// it again while the process runs.
    unsupported: Mutex<HashSet<(ProviderId, Lookup)>>,
    /// Ids their provider left out of its answer, with when: nothing is cached for them, and
    /// they are not asked about again before [`UNKNOWN_CONTENT_RETRY_AFTER`].
    unknown: Mutex<HashMap<(ProviderId, Lookup, String), Instant>>,
    /// Icons being fetched, by URL: a refresh that wants one already on its way waits for that
    /// fetch rather than starting another.
    icon_fetches: Mutex<HashMap<String, IconFetch>>,
    /// Icons that failed to arrive, with when: not fetched again before [`ICON_RETRY_AFTER`].
    icon_failures: Mutex<HashMap<String, Instant>>,
}

/// The provider and the provider's id of the project a file was installed from: the content id
/// recorded for the provider that updates the file. `None` for a file of no provider (added by
/// hand and not recognised).
///
/// Not `PackFile::installed_by`: that is the plan that installed the file, a modpack's own id for
/// every file of the pack.
fn content_key(file: &ContentFile) -> Option<(&ProviderId, &str)> {
    let provider_id = file.update_provider_id.as_ref()?;
    let info = file.update.as_ref()?.get(provider_id)?;
    Some((provider_id, info.content_id.as_str()))
}

/// The provider and the provider's id of the version a file was installed as, as
/// [`content_key`] finds its project. `None` for a file of no provider or of no recorded version.
fn version_key(file: &ContentFile) -> Option<(&ProviderId, &str)> {
    let provider_id = file.update_provider_id.as_ref()?;
    let info = file.update.as_ref()?.get(provider_id)?;
    (!info.version.is_empty()).then_some((provider_id, info.version.as_str()))
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
                refreshing: Mutex::new(HashSet::new()),
                unsupported: Mutex::new(HashSet::new()),
                unknown: Mutex::new(HashMap::new()),
                icon_fetches: Mutex::new(HashMap::new()),
                icon_failures: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Set each file's [`ContentFile::metadata`] and [`ContentFile::version_metadata`] to what
    /// the cache has for them, fresh or stale, and its [`ContentFile::icon_path`] to the cached
    /// icon of that metadata, if its file is there. Then start refreshing in the background what
    /// is missing or stale: the metadata of projects and versions first, then the icons — those
    /// of the metadata just fetched too.
    ///
    /// Reads only the caches. Once a refresh has brought anything, one
    /// [`InstanceEventType::Synced`] event of the instance tells the UI to list it again; a
    /// refresh that fails changes nothing and keeps the stale items and the icons there are.
    pub async fn attach(&self, instance_id: &str, files: &DashMap<String, ContentFile>) {
        let mut wanted: HashMap<ProviderId, Wanted> = HashMap::new();
        for file in files {
            if let Some((provider_id, content_id)) = content_key(&file) {
                wanted
                    .entry(provider_id.clone())
                    .or_default()
                    .projects
                    .insert(content_id.to_owned());
            }
            if let Some((provider_id, version_id)) = version_key(&file) {
                wanted
                    .entry(provider_id.clone())
                    .or_default()
                    .versions
                    .insert(version_id.to_owned());
            }
        }
        if wanted.is_empty() {
            return;
        }

        let now = SystemTime::now();
        let mut found = HashMap::new();
        let mut found_versions = HashMap::new();
        let mut due = Due::default();
        for (provider_id, wanted) in wanted {
            let content_ids: Vec<String> = wanted.projects.into_iter().collect();
            let cached = self.inner.cache.get_many(&provider_id, &content_ids).await;
            let stale = self
                .inner
                .due_ids(&provider_id, Lookup::Projects, content_ids, |id| {
                    cached.get(id).is_none_or(|item| item.is_stale(now))
                });
            if !stale.is_empty() {
                due.projects.insert(provider_id.clone(), stale);
            }

            let version_ids: Vec<String> = wanted.versions.into_iter().collect();
            let cached_versions = self
                .inner
                .cache
                .get_versions(&provider_id, &version_ids)
                .await;
            let stale = self
                .inner
                .due_ids(&provider_id, Lookup::Versions, version_ids, |id| {
                    cached_versions
                        .get(id)
                        .is_none_or(|item| item.is_stale(now))
                });
            if !stale.is_empty() {
                due.versions.insert(provider_id.clone(), stale);
            }

            found.insert(provider_id.clone(), cached);
            found_versions.insert(provider_id, cached_versions);
        }

        for mut file in files.iter_mut() {
            let metadata = content_key(&file).and_then(|(provider_id, content_id)| {
                Some(found.get(provider_id)?.get(content_id)?.metadata.clone())
            });
            let version_metadata = version_key(&file).and_then(|(provider_id, version_id)| {
                Some(
                    found_versions
                        .get(provider_id)?
                        .get(version_id)?
                        .metadata
                        .clone(),
                )
            });
            file.metadata = metadata;
            file.version_metadata = version_metadata;
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

    fn spawn_refresh(&self, instance_id: &str, due: Due, mut due_icons: HashSet<String>) {
        let Some(guard) = RefreshGuard::acquire(&self.inner, instance_id) else {
            return;
        };

        tokio::spawn(async move {
            let ((projects_updated, icon_urls), versions_updated) = futures::join!(
                guard.inner.refresh(due.projects),
                guard.inner.refresh_versions(due.versions),
            );
            let updated = projects_updated || versions_updated;
            let icons = guard.inner.lookup_icons(icon_urls).await;
            due_icons.extend(guard.inner.due_icons(icons));
            let icons_arrived = guard.inner.fetch_icons(due_icons).await;

            // Released before the event: the list it brings may need a refresh of its own.
            let inner = guard.inner.clone();
            let instance_id = guard.instance_id.clone();
            drop(guard);

            if updated || icons_arrived {
                inner
                    .event_emitter
                    .emit_safe(InstanceEvent {
                        event: InstanceEventType::Synced,
                        instance_id,
                    })
                    .await;
            }
        });
    }
}

/// An instance marked as being refreshed, until dropped — also when the refresh panics.
struct RefreshGuard {
    inner: Arc<Inner>,
    instance_id: String,
}

impl RefreshGuard {
    fn acquire(inner: &Arc<Inner>, instance_id: &str) -> Option<Self> {
        let started = inner
            .refreshing
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(instance_id.to_owned());

        started.then(|| Self {
            inner: inner.clone(),
            instance_id: instance_id.to_owned(),
        })
    }
}

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        self.inner
            .refreshing
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.instance_id);
    }
}

impl Inner {
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

    /// Of `ids`, those to ask `provider_id` about: missing or stale by `is_due`, and not left
    /// out of its answer lately. None when it cannot answer the lookup.
    fn due_ids(
        &self,
        provider_id: &ProviderId,
        lookup: Lookup,
        ids: Vec<String>,
        is_due: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        if self.is_unsupported(provider_id, lookup) {
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

    /// Note a failed lookup: a provider that cannot answer it is not asked it again; any other
    /// failure is only logged, and the next list asks again.
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
            err => tracing::warn!("Could not refresh {what} from {provider_id}: {err}"),
        }
    }

    /// Ask each provider about its due ids, all providers at once. Whether anything was cached,
    /// and the icon URLs of what was.
    async fn refresh(&self, due: HashMap<ProviderId, Vec<String>>) -> (bool, HashSet<String>) {
        let refreshed = futures::future::join_all(
            due.iter()
                .map(|(provider_id, content_ids)| self.refresh_of(provider_id, content_ids)),
        )
        .await;

        let updated = refreshed.iter().any(Option::is_some);
        let icon_urls = refreshed
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|(_, item)| item.icon_url)
            .collect();
        (updated, icon_urls)
    }

    /// The items of `content_ids` cached, paired with the ids they were cached under; `None`
    /// when nothing was.
    async fn refresh_of(
        &self,
        provider_id: &ProviderId,
        content_ids: &[String],
    ) -> Option<Vec<(String, ContentMetadata)>> {
        let provider = self.provider(provider_id).await?;

        match provider.get_contents_metadata(content_ids).await {
            Ok(items) => {
                let (keyed, unknown) = key_by_asked(content_ids, items);
                self.remember_unknown(provider_id, Lookup::Projects, unknown);
                if keyed.is_empty() {
                    return None;
                }
                self.cache.put_keyed(provider_id, &keyed).await;
                Some(keyed)
            }
            Err(err) => {
                self.lookup_failed(provider_id, Lookup::Projects, err);
                None
            }
        }
    }

    /// Ask each provider about its due version ids, all providers at once. Whether anything was
    /// cached.
    async fn refresh_versions(&self, due: HashMap<ProviderId, Vec<String>>) -> bool {
        futures::future::join_all(
            due.iter().map(|(provider_id, version_ids)| {
                self.refresh_versions_of(provider_id, version_ids)
            }),
        )
        .await
        .into_iter()
        .any(|updated| updated)
    }

    /// Whether any version of `version_ids` was cached.
    async fn refresh_versions_of(&self, provider_id: &ProviderId, version_ids: &[String]) -> bool {
        let Some(provider) = self.provider(provider_id).await else {
            return false;
        };

        match provider.get_versions_metadata(version_ids).await {
            Ok(items) => {
                let unknown = version_ids
                    .iter()
                    .filter(|id| !items.iter().any(|item| &item.version_id == *id))
                    .cloned()
                    .collect();
                self.remember_unknown(provider_id, Lookup::Versions, unknown);
                if items.is_empty() {
                    return false;
                }
                self.cache.put_versions(provider_id, &items).await;
                true
            }
            Err(err) => {
                self.lookup_failed(provider_id, Lookup::Versions, err);
                false
            }
        }
    }

    async fn lookup_icons(&self, urls: HashSet<String>) -> HashMap<String, CachedContentIcon> {
        futures::future::join_all(urls.into_iter().map(|url| async move {
            let icon = self.icons.lookup(&url).await;
            (url, icon)
        }))
        .await
        .into_iter()
        .collect()
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
    fn should_key_an_item_also_by_the_slug_it_was_asked_by() {
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
