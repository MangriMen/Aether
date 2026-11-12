use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
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
        &self,
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
                Ok::<_, PluginError>((Arc::new(archive), (self.clock)()))
            })
            .await;

        match result {
            Ok((archive, _)) => Ok(archive.clone()),
            Err(err) => {
                // Do not keep the empty slot, unless it is another request's by now.
                let mut slots = self.lock();
                if slots.get(url).is_some_and(|c| Arc::ptr_eq(c, &cell)) {
                    slots.remove(url);
                }
                Err(err)
            }
        }
    }

    /// The slot for `url`, made if there is none, after the expired ones are dropped and room is
    /// made for it.
    fn slot(&self, url: &str) -> Arc<OnceCell<Loaded>> {
        let now = (self.clock)();
        let mut slots = self.lock();

        slots.retain(|_, cell| {
            cell.get()
                .is_none_or(|(_, at)| now.saturating_duration_since(*at) < RELEASE_ARCHIVE_TTL)
        });
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
