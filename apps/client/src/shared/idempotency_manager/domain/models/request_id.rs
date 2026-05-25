use serde::{Deserialize, Serialize};
use specta_typescript::branded;

branded! {#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)] pub struct RequestId(String)}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
