use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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
    clearing: AtomicBool,
}

impl CacheStorageUseCase {
    pub fn new(cleaners: Vec<Arc<dyn CacheCleaner>>, activity: Arc<dyn StorageActivity>) -> Self {
        Self {
            cleaners,
            activity,
            clearing: AtomicBool::new(false),
        }
    }

    async fn is_busy(&self) -> bool {
        self.clearing.load(Ordering::SeqCst) || self.activity.is_busy().await
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
            busy: self.is_busy().await,
        }
    }
}

#[async_trait]
impl ClearCacheUseCasePort for CacheStorageUseCase {
    async fn execute(&self) -> ClearCacheOutcome {
        if self.clearing.swap(true, Ordering::SeqCst) {
            return ClearCacheOutcome::Busy;
        }
        let outcome = if self.activity.is_busy().await {
            ClearCacheOutcome::Busy
        } else {
            clear_caches(&self.cleaners).await;
            ClearCacheOutcome::Cleared
        };
        self.clearing.store(false, Ordering::SeqCst);
        outcome
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use crate::shared::housekeeping::CacheKind;

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
        async fn is_busy(&self) -> bool {
            self.0.load(Ordering::SeqCst)
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
