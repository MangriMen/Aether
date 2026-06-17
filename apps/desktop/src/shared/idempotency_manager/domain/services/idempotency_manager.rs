use std::{collections::HashSet, sync::Mutex};

use crate::shared::{IdempotencyManagerError, RequestId, idempotency_manager::ActiveRequest};

#[derive(Default)]
pub struct IdempotencyManager(Mutex<HashSet<RequestId>>);

impl IdempotencyManager {
    pub fn register(&self, id: RequestId) -> bool {
        let mut set = self.0.lock().unwrap();
        set.insert(id)
    }

    pub fn remove(&self, id: &RequestId) {
        if let Ok(mut set) = self.0.lock() {
            set.remove(id);
        }
    }
}

impl IdempotencyManager {
    pub fn lock_request(
        &self,
        id: RequestId,
    ) -> Result<ActiveRequest<'_>, IdempotencyManagerError> {
        if self.register(id.clone()) {
            Ok(ActiveRequest { manager: self, id })
        } else {
            Err(IdempotencyManagerError::DuplicateRequest { id })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_id(s: &str) -> RequestId {
        RequestId::new(s.to_string())
    }

    #[test]
    fn register_new_id_returns_true() {
        let manager = IdempotencyManager::default();
        assert!(manager.register(make_id("first")));
    }

    #[test]
    fn register_duplicate_id_returns_false() {
        let manager = IdempotencyManager::default();
        let id = make_id("dup");
        assert!(manager.register(id.clone()));
        assert!(!manager.register(id));
    }

    #[test]
    fn remove_existing_id_allows_reregister() {
        let manager = IdempotencyManager::default();
        let id = make_id("removable");
        assert!(manager.register(id.clone()));
        manager.remove(&id);
        assert!(
            manager.register(id),
            "After remove, should be able to register again"
        );
    }

    #[test]
    fn lock_request_new_id_returns_active_request() {
        let manager = IdempotencyManager::default();
        let result = manager.lock_request(make_id("new"));
        assert!(result.is_ok());
    }

    #[test]
    fn lock_request_duplicate_returns_error() {
        let manager = IdempotencyManager::default();
        let id = make_id("locked");
        let _first = manager.lock_request(id.clone()).unwrap();
        let result = manager.lock_request(id.clone());
        assert!(result.is_err());
        let Err(err) = result else { unreachable!() };
        assert_eq!(err.to_string(), format!("Duplicate request {id}"));
    }

    #[test]
    fn active_request_drop_releases_lock() {
        let manager = IdempotencyManager::default();
        let id = make_id("temp");

        {
            let _guard = manager.lock_request(id.clone()).unwrap();
            // Duplicate should fail while guard is alive
            assert!(manager.lock_request(id.clone()).is_err());
        }
        // After guard drops, should be able to lock again
        assert!(manager.lock_request(id).is_ok());
    }

    #[test]
    fn multiple_independent_ids_work() {
        let manager = IdempotencyManager::default();
        let id1 = make_id("alpha");
        let id2 = make_id("beta");

        let _guard1 = manager.lock_request(id1.clone()).unwrap();
        let _guard2 = manager.lock_request(id2.clone()).unwrap();
        // Each should be locked: duplicates fail
        assert!(manager.lock_request(id1.clone()).is_err());
        assert!(manager.lock_request(id2.clone()).is_err());
    }

    #[test]
    fn remove_nonexistent_id_does_nothing() {
        let manager = IdempotencyManager::default();
        let id = make_id("ghost");
        // Should not panic
        manager.remove(&id);
    }
}
