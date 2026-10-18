use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use tokio::sync::OwnedMutexGuard;

use crate::{
    features::instance::InstanceId,
    shared::housekeeping::{ActivityGate, ActivityLease},
};

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
/// Every lock also holds a lease of the shared [`ActivityGate`]. Pausing all of them at once
/// belongs to that gate, which the composition root hands in ([`Self::with_gate`]): while the
/// pause is held no instance can be locked, and it is only given when none is.
///
/// The lock is not reentrant: nothing that runs under it may take it again, directly or through
/// a plugin it calls.
pub struct InstanceLocks {
    /// Leased by every instance lock, paused by whoever empties the caches.
    gate: Arc<ActivityGate>,
    /// Dropped once nobody holds or waits for the lock, so the map does not keep every instance
    /// ever touched.
    locks: Mutex<HashMap<InstanceId, Weak<tokio::sync::Mutex<()>>>>,
}

/// The lock of one instance, held until dropped.
#[must_use = "the instance is unlocked as soon as the guard is dropped"]
pub struct InstanceLockGuard {
    _guard: OwnedMutexGuard<()>,
    _gate: ActivityLease,
}

impl Default for InstanceLocks {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceLocks {
    /// With a gate of its own, shared with nobody.
    pub fn new() -> Self {
        Self::with_gate(Arc::new(ActivityGate::new()))
    }

    pub fn with_gate(gate: Arc<ActivityGate>) -> Self {
        Self {
            gate,
            locks: Mutex::default(),
        }
    }

    /// Wait for the instance's lock.
    pub async fn lock(&self, instance_id: &InstanceId) -> InstanceLockGuard {
        let gate = self.gate.lease().await;
        InstanceLockGuard {
            _guard: self.lock_of(instance_id).lock_owned().await,
            _gate: gate,
        }
    }

    /// Wait for the instance's lock for at most `wait`: `None` if another still holds it then.
    pub async fn lock_within(
        &self,
        instance_id: &InstanceId,
        wait: std::time::Duration,
    ) -> Option<InstanceLockGuard> {
        tokio::time::timeout(wait, self.lock(instance_id))
            .await
            .ok()
    }

    /// The instance's lock, or `None` while another holds it.
    pub fn try_lock(&self, instance_id: &InstanceId) -> Option<InstanceLockGuard> {
        let gate = self.gate.try_lease()?;
        let guard = self.lock_of(instance_id).try_lock_owned().ok()?;
        Some(InstanceLockGuard {
            _guard: guard,
            _gate: gate,
        })
    }

    fn lock_of(&self, instance_id: &InstanceId) -> Arc<tokio::sync::Mutex<()>> {
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
        locks.insert(instance_id.clone(), Arc::downgrade(&lock));
        lock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(id: &str) -> InstanceId {
        InstanceId::try_from(id).unwrap()
    }

    #[tokio::test]
    async fn should_refuse_a_pause_when_an_instance_is_locked() {
        let gate = Arc::new(ActivityGate::new());
        let locks = InstanceLocks::with_gate(gate.clone());
        let guard = locks.lock(&id("a")).await;

        assert!(gate.try_pause().is_none());

        drop(guard);
        assert!(gate.try_pause().is_some());
    }

    #[tokio::test]
    async fn should_refuse_to_lock_when_paused() {
        let gate = Arc::new(ActivityGate::new());
        let locks = InstanceLocks::with_gate(gate.clone());
        let pause = gate.try_pause().unwrap();

        assert!(locks.try_lock(&id("a")).is_none());
        assert!(
            locks
                .lock_within(&id("a"), std::time::Duration::from_millis(20))
                .await
                .is_none()
        );

        drop(pause);
        assert!(locks.try_lock(&id("a")).is_some());
    }
}
