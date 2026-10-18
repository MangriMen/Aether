//! One holder at a time per key: a game version, a file.

use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex, PoisonError, Weak},
};

use tokio::sync::OwnedMutexGuard;

/// A lock per key, made on first use and dropped once nobody holds or waits for it, so the
/// map does not keep every key ever locked.
pub struct KeyedLocks<K> {
    state: Mutex<State<K>>,
}

struct State<K> {
    locks: HashMap<K, Weak<tokio::sync::Mutex<()>>>,
    /// The size to sweep the unused locks out at: twice what was left by the last sweep, so
    /// sweeping costs little per lock taken however many there are.
    sweep_at: usize,
}

/// A key's lock, held until dropped.
#[must_use = "the key is unlocked as soon as the guard is dropped"]
pub struct KeyedLockGuard {
    _guard: OwnedMutexGuard<()>,
}

/// Sweeping a map this small is not worth the bother.
const MIN_SWEEP: usize = 64;

impl<K> Default for KeyedLocks<K> {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                locks: HashMap::new(),
                sweep_at: MIN_SWEEP,
            }),
        }
    }
}

impl<K: Hash + Eq + Clone> KeyedLocks<K> {
    /// Wait for the lock of `key`.
    pub async fn lock(&self, key: &K) -> KeyedLockGuard {
        KeyedLockGuard {
            _guard: self.lock_of(key).lock_owned().await,
        }
    }

    fn lock_of(&self, key: &K) -> Arc<tokio::sync::Mutex<()>> {
        // Poisoning cannot leave the map inconsistent: every change to it is one call.
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        if let Some(lock) = state.locks.get(key).and_then(Weak::upgrade) {
            return lock;
        }
        if state.locks.len() >= state.sweep_at {
            state.locks.retain(|_, lock| lock.strong_count() > 0);
            state.sweep_at = (state.locks.len() * 2).max(MIN_SWEEP);
        }

        let lock = Arc::new(tokio::sync::Mutex::new(()));
        state.locks.insert(key.clone(), Arc::downgrade(&lock));
        lock
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .locks
            .len()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn should_make_a_second_holder_of_a_key_wait_and_not_one_of_another() {
        let locks = KeyedLocks::default();
        let held = locks.lock(&"a").await;

        let same = tokio::time::timeout(Duration::from_millis(30), locks.lock(&"a")).await;
        assert!(same.is_err(), "the key is held");
        let other = tokio::time::timeout(Duration::from_millis(30), locks.lock(&"b")).await;
        assert!(other.is_ok(), "another key is free");

        drop(held);
        let again = tokio::time::timeout(Duration::from_secs(1), locks.lock(&"a")).await;
        assert!(again.is_ok());
    }

    #[tokio::test]
    async fn should_not_keep_the_locks_nobody_holds() {
        let locks = KeyedLocks::default();
        for key in 0..1000 {
            drop(locks.lock(&key).await);
        }
        assert!(locks.len() <= 2 * MIN_SWEEP, "{}", locks.len());
    }
}
