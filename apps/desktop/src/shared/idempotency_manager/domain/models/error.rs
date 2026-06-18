use crate::shared::RequestId;

#[derive(Debug, thiserror::Error)]
pub enum IdempotencyManagerError {
    #[error("Duplicate request {id}")]
    DuplicateRequest { id: RequestId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_request_error_format() {
        let err = IdempotencyManagerError::DuplicateRequest {
            id: RequestId::new("req-1".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("Duplicate request"));
        assert!(msg.contains("req-1"));
    }

    #[test]
    fn duplicate_request_debug() {
        let err = IdempotencyManagerError::DuplicateRequest {
            id: RequestId::new("test".to_string()),
        };
        assert!(!format!("{err:?}").is_empty());
    }
}
