use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex, MutexGuard, PoisonError, Weak},
    time::{Duration, Instant},
};

use tokio::sync::OnceCell;

use crate::features::plugins::PluginError;

use super::DownloadedArchive;

/// How long a downloaded archive is kept for the next preview or install of the same release.
pub(super) const RELEASE_ARCHIVE_TTL: Duration = Duration::from_mins(10);

/// How many archives are kept at once. Past it the oldest is let go.
pub(super) const RELEASE_ARCHIVE_CAPACITY: usize = 4;

type Clock = Box<dyn Fn() -> Instant + Send + Sync>;

/// An archive and the moment it was downloaded.
type Loaded = (Arc<DownloadedArchive>, Instant);

type Slots = HashMap<String, Arc<OnceCell<Loaded>>>;

/// A short-lived cache of downloaded release archives, so that previewing another tag and back,
/// or installing right after the preview, does not download the same archive again.
///
/// Keyed by the archive's download URL. Only the archive is kept, never what is decided about it:
/// the size and host limits are those of the download, and the caller still checks the hash of
/// what it gets. A file lives while anyone holds its `Arc`, so an install that is under way keeps
/// its archive when the cache lets it go.
///
/// Requests for the same URL that overlap share one download. A download that fails is not kept.
/// An archive is dropped when it expires even if nobody asks for it again: each download starts a
/// task that sweeps the cache once the time to live is up.
pub struct ReleaseArchiveCache {
    slots: Mutex<Slots>,
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
            clock,
        }
    }

    /// The archive at `url`: the cached one, or the one `download` fetches.
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

        // A failure leaves the slot empty, for the requests waiting on it to download in their
        // turn into the same cell, and for the next request to drop when none is.
        result.map(|(archive, _)| archive.clone())
    }

    /// Drops the archives that have expired.
    pub(crate) fn sweep(&self) {
        let now = (self.clock)();
        retain_live(&mut self.lock(), now);
    }

    /// Sweeps once the time to live is up. The task holds the cache weakly, so it does not keep it
    /// alive.
    fn sweep_later(self: &Arc<Self>) {
        let cache: Weak<Self> = Arc::downgrade(self);
        tokio::spawn(async move {
            tokio::time::sleep(RELEASE_ARCHIVE_TTL).await;
            if let Some(cache) = cache.upgrade() {
                cache.sweep();
            }
        });
    }

    /// The slot for `url`, made if there is none, after the expired ones are dropped and room is
    /// made for it.
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

/// Keeps the archives that have not expired and the downloads that someone waits for. An empty
/// slot that only the map holds is a download that was given up.
fn retain_live(slots: &mut Slots, now: Instant) {
    slots.retain(|_, cell| match cell.get() {
        Some((_, at)) => now.saturating_duration_since(*at) < RELEASE_ARCHIVE_TTL,
        None => Arc::strong_count(cell) > 1,
    });
}
