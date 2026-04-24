use aether_core::features::events::ProgressBar;
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

use crate::features::events::ProgressEventTypeDto;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProgressBarDto {
    pub id: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    pub progress_type: ProgressEventTypeDto,
}

impl From<ProgressBar> for ProgressBarDto {
    fn from(value: ProgressBar) -> Self {
        Self {
            id: value.id,
            message: value.message,
            total: value.total,
            current: value.current,
            progress_type: value.progress_type.into(),
        }
    }
}
