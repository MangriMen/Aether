use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStatus {
    pub version: Option<String>,
    pub date: Option<DateTime<FixedOffset>>,
    pub body: Option<String>,
}
