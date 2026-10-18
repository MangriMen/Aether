use aether_core::features::events::ProgressBar;
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

use super::progress_event::bytes_to_number;
use crate::features::events::infra::{ProgressEventTypeDto, ProgressStageDto, ProgressStatusDto};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProgressBarDto {
    pub id: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    pub progress_type: ProgressEventTypeDto,
    pub stage: Option<ProgressStageDto>,
    pub status: ProgressStatusDto,
    pub parent_id: Option<Uuid>,
    /// As in `ProgressEventDto`.
    pub mirror_of: Option<Uuid>,
    /// A number, not `u64`, as in `ProgressEventDto`.
    pub bytes_done: Option<f64>,
    pub bytes_total: Option<f64>,
    /// Bytes per second.
    pub speed: Option<f64>,
}

impl From<ProgressBar> for ProgressBarDto {
    fn from(value: ProgressBar) -> Self {
        Self {
            id: value.id,
            message: value.message,
            total: value.total,
            current: value.current,
            progress_type: value.progress_type.into(),
            stage: value.stage.map(Into::into),
            status: value.status.into(),
            parent_id: value.parent_id,
            mirror_of: value.mirror_of,
            bytes_done: bytes_to_number(value.bytes_done),
            bytes_total: bytes_to_number(value.bytes_total),
            speed: value.speed,
        }
    }
}
