use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        events::{CacheEvent, EventEmitterExt, SharedEventEmitter},
        settings::{
            CacheKind, CacheUsage, CacheUsageEntry, ClearCacheOutcome,
            app::ports::{ClearCacheUseCasePort, GetCacheUsageUseCasePort, IsCacheBusyUseCasePort},
        },
    },
    shared::housekeeping::{CacheCleaner, StorageActivity, clear_caches},
};

/// What the user sees of the caches and may empty: the size of each, and one button to empty
/// them all — refused while the app is busy with something that uses them.
pub struct CacheStorageUseCase {
    caches: Vec<(CacheKind, Arc<dyn CacheCleaner>)>,
    activity: Arc<dyn StorageActivity>,
    event_emitter: SharedEventEmitter,
}

impl CacheStorageUseCase {
    pub fn new(
        caches: Vec<(CacheKind, Arc<dyn CacheCleaner>)>,
        activity: Arc<dyn StorageActivity>,
        event_emitter: SharedEventEmitter,
    ) -> Self {
        Self {
            caches,
            activity,
            event_emitter,
        }
    }
}

#[async_trait]
impl GetCacheUsageUseCasePort for CacheStorageUseCase {
    async fn execute(&self) -> CacheUsage {
        let entries =
            futures::future::join_all(self.caches.iter().map(|(kind, cleaner)| async move {
                CacheUsageEntry {
                    kind: *kind,
                    bytes: cleaner.size().await,
                }
            }))
            .await;

        CacheUsage { entries }
    }
}

#[async_trait]
impl IsCacheBusyUseCasePort for CacheStorageUseCase {
    async fn execute(&self) -> bool {
        self.activity.is_busy()
    }
}

#[async_trait]
impl ClearCacheUseCasePort for CacheStorageUseCase {
    async fn execute(&self) -> ClearCacheOutcome {
        // Held until the caches are empty: nothing starts using them meanwhile, and a second
        // clearing finds the app paused, so busy.
        let Some(pause) = self.activity.try_pause().await else {
            return ClearCacheOutcome::Busy;
        };
        clear_caches(self.caches.iter().map(|(_, cleaner)| cleaner)).await;
        // Released before the event: whoever asks again whether the app is busy must not find
        // this clearing in the way.
        drop(pause);
        self.event_emitter.emit_safe(CacheEvent::Changed).await;
        ClearCacheOutcome::Cleared
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    };

    use crate::features::events::{Event, EventEmitter, EventError};

    use crate::shared::housekeeping::ActivityPause;

    use super::*;
    use crate::shared::housekeeping::ActivityGate;

    struct Cleaner(AtomicU64);

    #[async_trait]
    impl CacheCleaner for Cleaner {
        fn what(&self) -> &'static str {
            "test"
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
        fn is_busy(&self) -> bool {
            self.0.load(Ordering::SeqCst)
        }

        async fn try_pause(&self) -> Option<ActivityPause> {
            (!self.0.load(Ordering::SeqCst)).then(|| Box::new(()) as ActivityPause)
        }
    }

    #[derive(Default)]
    struct Emitted(Mutex<Vec<Event>>);

    #[async_trait]
    impl EventEmitter<Event> for Emitted {
        async fn emit(&self, event: Event) -> Result<(), EventError> {
            self.0.lock().unwrap().push(event);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    fn fixture(busy: bool) -> (Arc<Cleaner>, Arc<Emitted>, CacheStorageUseCase) {
        let cleaner = Arc::new(Cleaner(AtomicU64::new(42)));
        let emitted = Arc::new(Emitted::default());
        let use_case = CacheStorageUseCase::new(
            vec![(CacheKind::Icons, cleaner.clone())],
            Arc::new(Activity(AtomicBool::new(busy))),
            emitted.clone(),
        );
        (cleaner, emitted, use_case)
    }

    #[tokio::test]
    async fn should_empty_the_caches_when_nothing_is_busy() {
        let (cleaner, emitted, use_case) = fixture(false);

        let outcome = ClearCacheUseCasePort::execute(&use_case).await;

        assert_eq!(outcome, ClearCacheOutcome::Cleared);
        assert_eq!(cleaner.size().await, 0);
        assert!(matches!(
            emitted.0.lock().unwrap().as_slice(),
            [Event::Cache(CacheEvent::Changed)]
        ));
    }

    #[tokio::test]
    async fn should_refuse_and_keep_the_caches_when_the_app_is_busy() {
        let (cleaner, emitted, use_case) = fixture(true);

        let outcome = ClearCacheUseCasePort::execute(&use_case).await;

        assert_eq!(outcome, ClearCacheOutcome::Busy);
        assert_eq!(cleaner.size().await, 42);
        assert!(emitted.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_report_the_sizes_when_asked_for_usage() {
        let (_, _, use_case) = fixture(true);

        let usage = GetCacheUsageUseCasePort::execute(&use_case).await;

        assert_eq!(
            usage.entries,
            [CacheUsageEntry {
                kind: CacheKind::Icons,
                bytes: 42
            }]
        );
    }

    #[tokio::test]
    async fn should_report_whether_clearing_is_refused_when_asked_if_busy() {
        assert!(IsCacheBusyUseCasePort::execute(&fixture(true).2).await);
        assert!(!IsCacheBusyUseCasePort::execute(&fixture(false).2).await);
    }

    #[tokio::test]
    async fn should_not_stop_the_next_clearing_when_asked_if_busy() {
        let gate = Arc::new(ActivityGate::new());
        let use_case = CacheStorageUseCase::new(
            Vec::new(),
            Arc::new(gate.clone()),
            Arc::new(Emitted::default()),
        );

        assert!(!IsCacheBusyUseCasePort::execute(&use_case).await);

        assert!(gate.try_pause().is_some());
    }
}
