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
