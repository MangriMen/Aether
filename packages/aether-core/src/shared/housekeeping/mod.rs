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

/// What a cache holds, as the user tells them apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheKind {
    /// Names, authors, versions and the like: the rows of `universal_cache`.
    Metadata,
    /// Icons of content.
    Icons,
    /// Downloaded modpack archives.
    ModpackArchives,
}

/// Whether the app is busy with something that uses the caches, so emptying them now would
/// pull files out from under it.
#[async_trait]
pub trait StorageActivity: Send + Sync {
    async fn is_busy(&self) -> bool;
}

/// A cache the user may empty whole. What the app needs of it is fetched again afterwards.
#[async_trait]
pub trait CacheCleaner: Send + Sync {
    /// What is emptied, for the logs.
    fn what(&self) -> &'static str;

    /// What the cache is, for the user.
    fn kind(&self) -> CacheKind;

    /// How many bytes the cache takes.
    async fn size(&self) -> u64;

    /// Empty the cache. How many items went.
    async fn clear(&self) -> u64;
}

/// The bytes of the files directly in `dir`; a missing or unreadable one counts for nothing.
pub async fn files_size(dir: &std::path::Path) -> u64 {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return 0;
    };

    let mut total = 0;
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(metadata) = entry.metadata().await
            && metadata.is_file()
        {
            total += metadata.len();
        }
    }
    total
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

/// Run `tasks` one after another in the background, logging what each removed.
pub fn spawn_housekeeping(tasks: Vec<Arc<dyn Housekeeping>>) {
    tokio::spawn(async move {
        for task in tasks {
            let removed = task.run().await;
            if removed > 0 {
                log::debug!("Housekeeping: removed {removed} of {}", task.what());
            }
        }
    });
}

/// Empty every cache of `cleaners`, one after another. What each removed, by what it is.
pub async fn clear_caches(cleaners: &[Arc<dyn CacheCleaner>]) -> Vec<(&'static str, u64)> {
    let mut cleared = Vec::with_capacity(cleaners.len());
    for cleaner in cleaners {
        let removed = cleaner.clear().await;
        log::info!("Cleared {removed} of {}", cleaner.what());
        cleared.push((cleaner.what(), removed));
    }
    cleared
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

        fn kind(&self) -> CacheKind {
            CacheKind::Icons
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

        let cleared = clear_caches(&[first.clone(), second.clone()]).await;

        assert_eq!(cleared, [("counting", 1), ("counting", 5)]);
    }
}
