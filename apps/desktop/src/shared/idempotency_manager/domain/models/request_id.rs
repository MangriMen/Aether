use serde::{Deserialize, Serialize};
use specta_typescript::branded;

branded! {#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)] pub struct RequestId(String)}

impl RequestId {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_display() {
        let id = RequestId::new("abc-123".to_string());
        assert_eq!(id.to_string(), "abc-123");
    }

    #[test]
    fn request_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let id = RequestId::new("unique".to_string());
        set.insert(id.clone());
        assert!(set.contains(&id));
    }

    #[test]
    fn request_id_different_ids_not_equal() {
        let a = RequestId::new("alpha".to_string());
        let b = RequestId::new("beta".to_string());
        assert_ne!(a, b);
    }

    #[test]
    fn request_id_serialize_deserialize() {
        let id = RequestId::new("test-id".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"test-id\"");
        let deserialized: RequestId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
