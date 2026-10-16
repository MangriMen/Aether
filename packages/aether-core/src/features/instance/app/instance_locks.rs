use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use tokio::sync::OwnedMutexGuard;

/// One change to an instance's content at a time.
///
/// Installing or updating content takes the instance's lock for the whole run, so two plans
/// never diff, write and roll back over each other's files; a second one waits for the first.
/// Removing, importing, and removing the instance itself take it for the whole operation and
/// wait as well, so none of them interleaves with a plan or rolls one back onto its result.
/// Enabling or disabling content waits only for a while (it is quick, and reachable from a
/// plugin); launching takes the lock only to start the game, and refuses to wait: an instance
/// whose content is being changed is still installing.
///
/// The lock is not reentrant: nothing that runs under it may take it again, directly or through
/// a plugin it calls.
#[derive(Default)]
pub struct InstanceLocks {
    /// Dropped once nobody holds or waits for the lock, so the map does not keep every instance
    /// ever touched.
    locks: Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>,
}

/// The lock of one instance, held until dropped.
#[must_use = "the instance is unlocked as soon as the guard is dropped"]
pub struct InstanceLockGuard {
    _guard: OwnedMutexGuard<()>,
}

impl InstanceLocks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Wait for the instance's lock.
    pub async fn lock(&self, instance_id: &str) -> InstanceLockGuard {
        InstanceLockGuard {
            _guard: self.lock_of(instance_id).lock_owned().await,
        }
    }

    /// Wait for the instance's lock for at most `wait`: `None` if another still holds it then.
    pub async fn lock_within(
        &self,
        instance_id: &str,
        wait: std::time::Duration,
    ) -> Option<InstanceLockGuard> {
        tokio::time::timeout(wait, self.lock_of(instance_id).lock_owned())
            .await
            .ok()
            .map(|guard| InstanceLockGuard { _guard: guard })
    }

    /// The instance's lock, or `None` while another holds it.
    pub fn try_lock(&self, instance_id: &str) -> Option<InstanceLockGuard> {
        self.lock_of(instance_id)
            .try_lock_owned()
            .ok()
            .map(|guard| InstanceLockGuard { _guard: guard })
    }

    /// Whether any instance's lock is held, or waited for.
    pub fn is_any_locked(&self) -> bool {
        let locks = self
            .locks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        locks.values().any(|lock| lock.strong_count() > 0)
    }

    fn lock_of(&self, instance_id: &str) -> Arc<tokio::sync::Mutex<()>> {
        // Poisoning cannot leave the map inconsistent: every change to it is one call.
        let mut locks = self
            .locks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        locks.retain(|_, lock| lock.strong_count() > 0);

        if let Some(lock) = locks.get(instance_id).and_then(Weak::upgrade) {
            return lock;
        }
        let lock = Arc::new(tokio::sync::Mutex::new(()));
        locks.insert(instance_id.to_owned(), Arc::downgrade(&lock));
        lock
    }
}
