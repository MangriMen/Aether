use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStatus {
    pub version: Option<String>,
    pub date: Option<String>,
    pub body: Option<String>,
}
