use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub fraction: Option<f64>,
    pub phase: UpdatePhase,
    pub version: String,
    pub current_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase {
    Started,
    Progress,
    Finished,
}
