use crate::shared::RequestId;

#[derive(Debug, thiserror::Error)]
pub enum IdempotencyManagerError {
    #[error("Duplicate request {id}")]
    DuplicateRequest { id: RequestId },
}
