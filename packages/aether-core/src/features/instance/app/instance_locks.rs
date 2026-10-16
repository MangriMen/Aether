use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use tokio::sync::{OwnedMutexGuard, OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

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
/// All of them can be paused at once ([`Self::try_pause`]): while the pause is held no instance
/// can be locked, and it is only given when none is.
///
/// The lock is not reentrant: nothing that runs under it may take it again, directly or through
/// a plugin it calls.
#[derive(Default)]
pub struct InstanceLocks {
    /// Read-held by every instance lock, write-held by a pause.
    gate: Arc<RwLock<()>>,
    /// Dropped once nobody holds or waits for the lock, so the map does not keep every instance
    /// ever touched.
    locks: Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>,
}

/// The lock of one instance, held until dropped.
#[must_use = "the instance is unlocked as soon as the guard is dropped"]
pub struct InstanceLockGuard {
    _guard: OwnedMutexGuard<()>,
    _gate: OwnedRwLockReadGuard<()>,
}

/// No instance's content is being changed, and none can start, until this is dropped.
#[must_use = "instances are unpaused as soon as this is dropped"]
pub struct InstanceLocksPause {
    _gate: OwnedRwLockWriteGuard<()>,
}

impl InstanceLocks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Wait for the instance's lock.
    pub async fn lock(&self, instance_id: &str) -> InstanceLockGuard {
        let gate = self.gate.clone().read_owned().await;
        InstanceLockGuard {
            _guard: self.lock_of(instance_id).lock_owned().await,
            _gate: gate,
        }
    }

    /// Wait for the instance's lock for at most `wait`: `None` if another still holds it then.
    pub async fn lock_within(
        &self,
        instance_id: &str,
        wait: std::time::Duration,
    ) -> Option<InstanceLockGuard> {
        tokio::time::timeout(wait, self.lock(instance_id))
            .await
            .ok()
    }

    /// The instance's lock, or `None` while another holds it.
    pub fn try_lock(&self, instance_id: &str) -> Option<InstanceLockGuard> {
        let gate = self.gate.clone().try_read_owned().ok()?;
        let guard = self.lock_of(instance_id).try_lock_owned().ok()?;
        Some(InstanceLockGuard {
            _guard: guard,
            _gate: gate,
        })
    }

    /// Stop every instance from being locked, or `None` while any is (or is waited for).
    pub fn try_pause(&self) -> Option<InstanceLocksPause> {
        self.gate
            .clone()
            .try_write_owned()
            .ok()
            .map(|gate| InstanceLocksPause { _gate: gate })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_refuse_a_pause_while_an_instance_is_locked() {
        let locks = InstanceLocks::new();
        let guard = locks.lock("a").await;

        assert!(locks.try_pause().is_none());

        drop(guard);
        assert!(locks.try_pause().is_some());
    }

    #[tokio::test]
    async fn should_refuse_to_lock_while_paused() {
        let locks = InstanceLocks::new();
        let pause = locks.try_pause().unwrap();

        assert!(locks.try_lock("a").is_none());
        assert!(
            locks
                .lock_within("a", std::time::Duration::from_millis(20))
                .await
                .is_none()
        );

        drop(pause);
        assert!(locks.try_lock("a").is_some());
    }
}
