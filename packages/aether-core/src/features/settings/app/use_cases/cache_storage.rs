use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::settings::{
        CacheUsage, CacheUsageEntry, ClearCacheOutcome,
        app::ports::{ClearCacheUseCasePort, GetCacheUsageUseCasePort},
    },
    shared::housekeeping::{CacheCleaner, StorageActivity, clear_caches},
};

/// What the user sees of the caches and may empty: the size of each, and one button to empty
/// them all — refused while the app is busy with something that uses them.
pub struct CacheStorageUseCase {
    cleaners: Vec<Arc<dyn CacheCleaner>>,
    activity: Arc<dyn StorageActivity>,
}

impl CacheStorageUseCase {
    pub fn new(cleaners: Vec<Arc<dyn CacheCleaner>>, activity: Arc<dyn StorageActivity>) -> Self {
        Self { cleaners, activity }
    }
}

#[async_trait]
impl GetCacheUsageUseCasePort for CacheStorageUseCase {
    async fn execute(&self) -> CacheUsage {
        let mut entries = Vec::with_capacity(self.cleaners.len());
        for cleaner in &self.cleaners {
            entries.push(CacheUsageEntry {
                kind: cleaner.kind(),
                bytes: cleaner.size().await,
            });
        }

        CacheUsage {
            entries,
            busy: self.activity.try_pause().await.is_none(),
        }
    }
}

#[async_trait]
impl ClearCacheUseCasePort for CacheStorageUseCase {
    async fn execute(&self) -> ClearCacheOutcome {
        // Held until the caches are empty: nothing starts using them meanwhile, and a second
        // clearing finds the app paused, so busy.
        let Some(_pause) = self.activity.try_pause().await else {
            return ClearCacheOutcome::Busy;
        };
        clear_caches(&self.cleaners).await;
        ClearCacheOutcome::Cleared
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    use crate::shared::housekeeping::{ActivityPause, CacheKind};

    use super::*;

    struct Cleaner(AtomicU64);

    #[async_trait]
    impl CacheCleaner for Cleaner {
        fn what(&self) -> &'static str {
            "test"
        }

        fn kind(&self) -> CacheKind {
            CacheKind::Icons
        }

        async fn size(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }

        async fn clear(&self) -> u64 {
            self.0.swap(0, Ordering::SeqCst)
        }
    }

    struct Activity(AtomicBool);

    #[async_trait]
    impl StorageActivity for Activity {
        async fn try_pause(&self) -> Option<ActivityPause> {
            (!self.0.load(Ordering::SeqCst)).then(|| Box::new(()) as ActivityPause)
        }
    }

    fn fixture(busy: bool) -> (Arc<Cleaner>, CacheStorageUseCase) {
        let cleaner = Arc::new(Cleaner(AtomicU64::new(42)));
        let use_case = CacheStorageUseCase::new(
            vec![cleaner.clone()],
            Arc::new(Activity(AtomicBool::new(busy))),
        );
        (cleaner, use_case)
    }

    #[tokio::test]
    async fn should_empty_the_caches_when_nothing_is_busy() {
        let (cleaner, use_case) = fixture(false);

        let outcome = ClearCacheUseCasePort::execute(&use_case).await;

        assert_eq!(outcome, ClearCacheOutcome::Cleared);
        assert_eq!(cleaner.size().await, 0);
    }

    #[tokio::test]
    async fn should_refuse_and_keep_the_caches_when_the_app_is_busy() {
        let (cleaner, use_case) = fixture(true);

        let outcome = ClearCacheUseCasePort::execute(&use_case).await;

        assert_eq!(outcome, ClearCacheOutcome::Busy);
        assert_eq!(cleaner.size().await, 42);
    }

    #[tokio::test]
    async fn should_report_the_sizes_and_whether_clearing_is_refused_when_asked_for_usage() {
        let (_, use_case) = fixture(true);

        let usage = GetCacheUsageUseCasePort::execute(&use_case).await;

        assert_eq!(
            usage.entries,
            [CacheUsageEntry {
                kind: CacheKind::Icons,
                bytes: 42
            }]
        );
        assert!(usage.busy);
    }
}
