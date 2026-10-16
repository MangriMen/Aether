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

/// A cache the user may empty whole. What the app needs of it is fetched again afterwards.
#[async_trait]
pub trait CacheCleaner: Send + Sync {
    /// What is emptied, for the logs.
    fn what(&self) -> &'static str;

    /// Empty the cache. How many items went.
    async fn clear(&self) -> u64;
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

        async fn clear(&self) -> u64 {
            self.0.fetch_add(1, Ordering::SeqCst) + 1
        }
    }

    #[tokio::test]
    async fn should_clear_every_cache_and_report_each() {
        let first = Arc::new(Counting(AtomicU64::new(0)));
        let second = Arc::new(Counting(AtomicU64::new(4)));

        let cleared = clear_caches(&[first.clone(), second.clone()]).await;

        assert_eq!(cleared, [("counting", 1), ("counting", 5)]);
    }
}
