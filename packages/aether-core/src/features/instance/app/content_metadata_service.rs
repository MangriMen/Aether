//! What an instance's content list shows a file by — its project's name, description, icon —
//! taken from [`ContentMetadataCache`] and refreshed from the providers behind the list's back.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant, SystemTime},
};

use dashmap::DashMap;

use crate::{
    features::{
        events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
        instance::{
            ContentFile, ContentMetadata, ContentMetadataCache, ContentProvider, InstanceError,
            ProviderId,
        },
    },
    shared::capability::domain::CapabilityRegistry,
};

/// How long an id its provider did not know is left alone before it is asked about again.
const UNKNOWN_CONTENT_RETRY_AFTER: Duration = Duration::from_mins(30);

/// Puts the cached [`ContentMetadata`] of their projects on an instance's content files, and
/// asks the providers for what is missing or stale without making the list wait.
///
/// One per process: it remembers which instances are being refreshed, which providers cannot
/// describe their content, and which ids their provider did not know.
pub struct ContentMetadataService {
    inner: Arc<Inner>,
}

struct Inner {
    cache: Arc<dyn ContentMetadataCache>,
    providers: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    event_emitter: SharedEventEmitter,
    /// Instances with a refresh running: a list opened meanwhile starts no second one.
    refreshing: Mutex<HashSet<String>>,
    /// Providers that answered [`InstanceError::UnsupportedOperation`]: not asked again while
    /// the process runs.
    unsupported: Mutex<HashSet<ProviderId>>,
    /// Ids their provider left out of its answer, with when: nothing is cached for them, and
    /// they are not asked about again before [`UNKNOWN_CONTENT_RETRY_AFTER`].
    unknown: Mutex<HashMap<(ProviderId, String), Instant>>,
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

impl ContentMetadataService {
    pub fn new(
        cache: Arc<dyn ContentMetadataCache>,
        providers: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        event_emitter: SharedEventEmitter,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                cache,
                providers,
                event_emitter,
                refreshing: Mutex::new(HashSet::new()),
                unsupported: Mutex::new(HashSet::new()),
                unknown: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Set each file's [`ContentFile::metadata`] to what the cache has for it, fresh or stale,
    /// and start refreshing in the background what is missing or stale.
    ///
    /// Reads only the cache. Once a refresh has cached anything, an
    /// [`InstanceEventType::Synced`] event of the instance tells the UI to list it again; a
    /// refresh that fails changes nothing and keeps the stale items.
    pub async fn attach(&self, instance_id: &str, files: &DashMap<String, ContentFile>) {
        let mut wanted: HashMap<ProviderId, HashSet<String>> = HashMap::new();
        for file in files {
            if let Some((provider_id, content_id)) = content_key(&file) {
                wanted
                    .entry(provider_id.clone())
                    .or_default()
                    .insert(content_id.to_owned());
            }
        }
        if wanted.is_empty() {
            return;
        }

        let now = SystemTime::now();
        let mut found = HashMap::new();
        let mut due = HashMap::new();
        for (provider_id, content_ids) in wanted {
            let content_ids: Vec<String> = content_ids.into_iter().collect();
            let cached = self.inner.cache.get_many(&provider_id, &content_ids).await;

            if !self.inner.is_unsupported(&provider_id) {
                let stale: Vec<String> = content_ids
                    .into_iter()
                    .filter(|id| cached.get(id).is_none_or(|item| item.is_stale(now)))
                    .filter(|id| !self.inner.is_known_unknown(&provider_id, id))
                    .collect();
                if !stale.is_empty() {
                    due.insert(provider_id.clone(), stale);
                }
            }
            found.insert(provider_id, cached);
        }

        for mut file in files.iter_mut() {
            let metadata = content_key(&file).and_then(|(provider_id, content_id)| {
                Some(found.get(provider_id)?.get(content_id)?.metadata.clone())
            });
            file.metadata = metadata;
        }

        if !due.is_empty() {
            self.spawn_refresh(instance_id, due);
        }
    }

    fn spawn_refresh(&self, instance_id: &str, due: HashMap<ProviderId, Vec<String>>) {
        let Some(guard) = RefreshGuard::acquire(&self.inner, instance_id) else {
            return;
        };

        tokio::spawn(async move {
            let updated = guard.inner.refresh(due).await;
            // Released before the event: the list it brings may need a refresh of its own.
            let inner = guard.inner.clone();
            let instance_id = guard.instance_id.clone();
            drop(guard);

            if updated {
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
    fn is_unsupported(&self, provider_id: &ProviderId) -> bool {
        self.unsupported
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(provider_id)
    }

    fn is_known_unknown(&self, provider_id: &ProviderId, content_id: &str) -> bool {
        self.unknown
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(provider_id.clone(), content_id.to_owned()))
            .is_some_and(|at| at.elapsed() < UNKNOWN_CONTENT_RETRY_AFTER)
    }

    fn remember_unknown(&self, provider_id: &ProviderId, content_ids: Vec<String>) {
        let now = Instant::now();
        let mut unknown = self.unknown.lock().unwrap_or_else(PoisonError::into_inner);
        unknown.retain(|_, at| at.elapsed() < UNKNOWN_CONTENT_RETRY_AFTER);
        unknown.extend(
            content_ids
                .into_iter()
                .map(|content_id| ((provider_id.clone(), content_id), now)),
        );
    }

    /// Ask each provider about its due ids, all providers at once. Whether anything was cached.
    async fn refresh(&self, due: HashMap<ProviderId, Vec<String>>) -> bool {
        let refreshed = futures::future::join_all(
            due.iter()
                .map(|(provider_id, content_ids)| self.refresh_of(provider_id, content_ids)),
        )
        .await;

        refreshed.into_iter().any(|updated| updated)
    }

    async fn refresh_of(&self, provider_id: &ProviderId, content_ids: &[String]) -> bool {
        let Ok(entry) = self
            .providers
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
        else {
            // Its plugin is off or gone: the file is shown by what it has.
            tracing::debug!("No content provider {provider_id} to describe its content");
            return false;
        };

        match entry.capability.get_contents_metadata(content_ids).await {
            Ok(items) => {
                let (keyed, unknown) = key_by_asked(content_ids, items);
                self.remember_unknown(provider_id, unknown);
                if keyed.is_empty() {
                    return false;
                }
                self.cache.put_keyed(provider_id, &keyed).await;
                true
            }
            Err(InstanceError::UnsupportedOperation { .. }) => {
                tracing::debug!("Content provider {provider_id} does not describe its content");
                self.unsupported
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(provider_id.clone());
                false
            }
            Err(err) => {
                tracing::warn!("Could not refresh content metadata from {provider_id}: {err}");
                false
            }
        }
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
