use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct UpdateProgress {
    pub fraction: Option<f64>,
    pub phase: UpdatePhase,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase {
    Started,
    Progress,
    Finished,
}
