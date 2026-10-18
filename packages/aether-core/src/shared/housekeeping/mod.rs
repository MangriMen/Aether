//! Upkeep of the caches: what runs by itself in the background ([`Housekeeping`]) and what the
//! user empties by hand ([`CacheCleaner`]).
//!
//! The principle: whatever is still used is never cleared automatically; the rest is cleared
//! by the user.

use std::sync::Arc;

use async_trait::async_trait;

/// A cleanup that runs by itself, in the background, after startup. Its thresholds are its own,
/// set where it is built.
#[async_trait]
pub trait Housekeeping: Send + Sync {
    /// What is cleaned, for the logs.
    fn what(&self) -> &'static str;

    /// Clean once. How many items went.
    async fn run(&self) -> u64;
}

/// Holds the app idle, as far as the caches go, until dropped.
pub type ActivityPause = Box<dyn std::any::Any + Send>;

/// Whether the app is busy with something that uses the caches, so emptying them now would
/// pull files out from under it.
#[async_trait]
pub trait StorageActivity: Send + Sync {
    /// Stop such work from starting, or `None` when it is under way already. Checking and
    /// stopping are one step, so nothing slips in between.
    async fn try_pause(&self) -> Option<ActivityPause>;
}

/// A cache the user may empty whole. What the app needs of it is fetched again afterwards.
#[async_trait]
pub trait CacheCleaner: Send + Sync {
    /// What is emptied, for the logs.
    fn what(&self) -> &'static str;

    /// How many bytes the cache takes.
    async fn size(&self) -> u64;

    /// Empty the cache. How many items went.
    async fn clear(&self) -> u64;
}

/// The bytes of every file under `dir`, at any depth. Links are not followed; a missing or
/// unreadable folder counts for nothing.
pub async fn dir_size(dir: &std::path::Path) -> u64 {
    let dir = dir.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut total = 0;
        let mut pending = vec![dir];
        while let Some(dir) = pending.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if file_type.is_dir() {
                    pending.push(entry.path());
                } else if file_type.is_file()
                    && let Ok(metadata) = entry.metadata()
                {
                    total += metadata.len();
                }
            }
        }
        total
    })
    .await
    .unwrap_or_default()
}

/// Remove the files directly in `dir` that `should_remove` accepts; folders and links are left
/// alone, a missing folder counts for nothing. Which went.
pub async fn remove_files(
    dir: &std::path::Path,
    should_remove: impl Fn(&std::path::Path) -> bool + Send + 'static,
) -> Vec<std::path::PathBuf> {
    let dir = dir.to_owned();
    tokio::task::spawn_blocking(move || {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Vec::new();
        };
        let mut removed = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !entry.file_type().is_ok_and(|file_type| file_type.is_file())
                || !should_remove(&path)
            {
                continue;
            }
            match std::fs::remove_file(&path) {
                Ok(()) => removed.push(path),
                Err(err) => tracing::debug!("Failed to remove {}: {err}", path.display()),
            }
        }
        removed
    })
    .await
    .unwrap_or_default()
}

/// Run each of `tasks` in the background, side by side, logging what each removed. `on_removed`
/// is told each time something went.
pub fn spawn_housekeeping(
    tasks: Vec<Arc<dyn Housekeeping>>,
    on_removed: &Arc<dyn Fn() + Send + Sync>,
) {
    for task in tasks {
        let on_removed = on_removed.clone();
        tokio::spawn(async move {
            let removed = task.run().await;
            if removed > 0 {
                log::debug!("Housekeeping: removed {removed} of {}", task.what());
                on_removed();
            }
        });
    }
}

/// Empty every cache of `cleaners` side by side, as they are independent. What each removed, by
/// what it is, in the order given.
pub async fn clear_caches<'a>(
    cleaners: impl IntoIterator<Item = &'a Arc<dyn CacheCleaner>>,
) -> Vec<(&'static str, u64)> {
    futures::future::join_all(cleaners.into_iter().map(|cleaner| async move {
        let removed = cleaner.clear().await;
        log::info!("Cleared {removed} of {}", cleaner.what());
        (cleaner.what(), removed)
    }))
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    struct Counting(AtomicU64);

    #[async_trait]
    impl CacheCleaner for Counting {
        fn what(&self) -> &'static str {
            "counting"
        }

        async fn size(&self) -> u64 {
            0
        }

        async fn clear(&self) -> u64 {
            self.0.fetch_add(1, Ordering::SeqCst) + 1
        }
    }

    #[tokio::test]
    async fn should_sum_the_files_at_any_depth_when_measuring_a_folder() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a"), [0; 10]).unwrap();
        std::fs::create_dir_all(dir.path().join("x/y")).unwrap();
        std::fs::write(dir.path().join("x/b"), [0; 5]).unwrap();
        std::fs::write(dir.path().join("x/y/c"), [0; 1]).unwrap();

        assert_eq!(dir_size(dir.path()).await, 16);
        assert_eq!(dir_size(&dir.path().join("missing")).await, 0);
    }

    #[tokio::test]
    async fn should_clear_every_cache_and_report_each() {
        let first = Arc::new(Counting(AtomicU64::new(0)));
        let second = Arc::new(Counting(AtomicU64::new(4)));

        let cleared = clear_caches(&[first.clone() as _, second.clone() as _]).await;

        assert_eq!(cleared, [("counting", 1), ("counting", 5)]);
    }
}
