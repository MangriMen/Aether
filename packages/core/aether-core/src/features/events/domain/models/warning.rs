use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct WarningEvent {
    pub message: String,
}
