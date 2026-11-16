use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex, MutexGuard, PoisonError, Weak},
    time::{Duration, Instant},
};

use tokio::sync::OnceCell;

use crate::features::plugins::{PluginError, ProviderPluginPreview};

use super::DownloadedArchive;

pub(super) const RELEASE_ARCHIVE_TTL: Duration = Duration::from_mins(10);

pub(super) const RELEASE_ARCHIVE_CAPACITY: usize = 4;

/// Short, so a release published a moment ago shows up soon, and enough for the list, the
/// preview of a tag and the install that follow one another.
pub(super) const RELEASE_LISTING_TTL: Duration = Duration::from_secs(60);

pub(super) const RELEASE_LISTING_CAPACITY: usize = 8;

type Clock = Box<dyn Fn() -> Instant + Send + Sync>;

/// An archive and the moment it was downloaded.
type Loaded = (Arc<DownloadedArchive>, Instant);

type Slots = HashMap<String, Arc<OnceCell<Loaded>>>;

type Listings = HashMap<String, (Arc<ProviderPluginPreview>, Instant)>;

/// Keeps downloaded archives for a short time so a preview and the install after it share one
/// download. Keyed by URL; only the file is cached, the caller still checks the hash. A file lives
/// while anyone holds its `Arc`.
///
/// Also keeps the release listing of a plugin for a shorter time, so listing, previewing a tag
/// and installing do not each ask the provider (GitHub allows few requests without a token). A
/// stale listing does no harm: the install is held to the hash of what was previewed.
pub struct ReleaseArchiveCache {
    slots: Mutex<Slots>,
    listings: Mutex<Listings>,
    clock: Clock,
}

impl Default for ReleaseArchiveCache {
    fn default() -> Self {
        Self::with_clock(Box::new(Instant::now))
    }
}

impl ReleaseArchiveCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_clock(clock: Clock) -> Self {
        Self {
            slots: Mutex::new(HashMap::new()),
            listings: Mutex::new(HashMap::new()),
            clock,
        }
    }

    pub(crate) async fn get_or_download<F, Fut>(
        self: &Arc<Self>,
        url: &str,
        download: F,
    ) -> Result<Arc<DownloadedArchive>, PluginError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<DownloadedArchive, PluginError>>,
    {
        let cell = self.slot(url);
        let result = cell
            .get_or_try_init(|| async {
                let archive = download().await?;
                self.sweep_later();
                Ok::<_, PluginError>((Arc::new(archive), (self.clock)()))
            })
            .await;

        // On failure the slot stays empty: waiting requests retry in the same cell, otherwise the
        // next sweep drops it.
        result.map(|(archive, _)| archive.clone())
    }

    /// The listing under `key`, fetched when there is none younger than the TTL. Concurrent
    /// callers may each fetch: a listing is cheap next to an archive.
    pub(crate) async fn get_or_fetch_listing<F, Fut>(
        &self,
        key: &str,
        fetch: F,
    ) -> Result<Arc<ProviderPluginPreview>, PluginError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<ProviderPluginPreview, PluginError>>,
    {
        if let Some(listing) = self.cached_listing(key) {
            return Ok(listing);
        }

        let listing = Arc::new(fetch().await?);
        let now = (self.clock)();
        let mut listings = self.listings.lock().unwrap_or_else(PoisonError::into_inner);

        listings.retain(|_, (_, at)| now.saturating_duration_since(*at) < RELEASE_LISTING_TTL);
        if listings.len() >= RELEASE_LISTING_CAPACITY
            && let Some(oldest) = listings
                .iter()
                .min_by_key(|(_, (_, at))| *at)
                .map(|(key, _)| key.clone())
        {
            listings.remove(&oldest);
        }
        listings.insert(key.to_owned(), (listing.clone(), now));

        Ok(listing)
    }

    fn cached_listing(&self, key: &str) -> Option<Arc<ProviderPluginPreview>> {
        let now = (self.clock)();
        let listings = self.listings.lock().unwrap_or_else(PoisonError::into_inner);

        listings
            .get(key)
            .filter(|(_, at)| now.saturating_duration_since(*at) < RELEASE_LISTING_TTL)
            .map(|(listing, _)| listing.clone())
    }

    pub(crate) fn sweep(&self) {
        let now = (self.clock)();
        retain_live(&mut self.lock(), now);
    }

    /// The task holds the cache weakly so it does not keep it alive.
    fn sweep_later(self: &Arc<Self>) {
        let cache: Weak<Self> = Arc::downgrade(self);
        tokio::spawn(async move {
            tokio::time::sleep(RELEASE_ARCHIVE_TTL).await;
            if let Some(cache) = cache.upgrade() {
                cache.sweep();
            }
        });
    }

    fn slot(&self, url: &str) -> Arc<OnceCell<Loaded>> {
        let now = (self.clock)();
        let mut slots = self.lock();

        retain_live(&mut slots, now);
        if let Some(cell) = slots.get(url) {
            return cell.clone();
        }

        if slots.len() >= RELEASE_ARCHIVE_CAPACITY {
            // A download in flight counts as the newest.
            let oldest = slots
                .iter()
                .min_by_key(|(_, cell)| cell.get().map_or(now, |(_, at)| *at))
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest {
                slots.remove(&key);
            }
        }
        let cell = Arc::new(OnceCell::new());
        slots.insert(url.to_owned(), cell.clone());
        cell
    }

    fn lock(&self) -> MutexGuard<'_, Slots> {
        // The map is consistent after every step, so a poisoned lock is still good to use.
        self.slots.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

// An empty slot only the map holds is an abandoned download.
fn retain_live(slots: &mut Slots, now: Instant) {
    slots.retain(|_, cell| match cell.get() {
        Some((_, at)) => now.saturating_duration_since(*at) < RELEASE_ARCHIVE_TTL,
        None => Arc::strong_count(cell) > 1,
    });
}
