use crate::shared::{RequestId, idempotency_manager::IdempotencyManager};

pub struct ActiveRequest<'a> {
    pub(crate) manager: &'a IdempotencyManager,
    pub id: RequestId,
}

impl Drop for ActiveRequest<'_> {
    fn drop(&mut self) {
        self.manager.remove(&self.id);
    }
}
