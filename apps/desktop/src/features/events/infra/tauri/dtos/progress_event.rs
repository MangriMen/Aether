use std::path::PathBuf;

use aether_core::features::events::{
    CancelState, ProgressEvent, ProgressEventType, ProgressStage, ProgressStatus,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;

use crate::core::FrontendError;
use crate::features::update::{UpdatePhase, UpdateProgress};

#[derive(Debug, Clone, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEventDto {
    /// The number of the event, from one counter for all the events the backend sends: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    pub event: ProgressEventTypeDto,
    pub progress_bar_id: Uuid,
    pub fraction: Option<f64>, // None means the loading is over; `status` tells how it ended
    pub message: String,
    pub stage: Option<ProgressStageDto>,
    pub status: ProgressStatusDto,
    pub parent_id: Option<Uuid>,
    /// The shared bar this one mirrors (a download it waits on), if it is a mirror: it shows
    /// what that bar shows and its bytes are not its parent's.
    pub mirror_of: Option<Uuid>,
    /// A number, not `u64`: specta refuses to export `u64`, and a download never nears 2^53 bytes.
    pub bytes_done: Option<f64>,
    /// A number, not `u64`, like `bytes_done`.
    pub bytes_total: Option<f64>,
    /// Bytes per second.
    pub speed: Option<f64>,
    /// Whether the user may call the bar's work off (`cancel_progress`); absent for work that
    /// cannot be.
    pub cancel: Option<CancelStateDto>,
}

/// The bytes as a number the frontend reads: see [`ProgressEventDto::bytes_done`].
#[allow(clippy::cast_precision_loss)]
pub(super) fn bytes_to_number(bytes: Option<u64>) -> Option<f64> {
    bytes.map(|bytes| bytes as f64)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStageDto {
    Preparing,
    Downloading,
    Verifying,
    Installing,
    Extracting,
    Finishing,
    ByPlugin,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum CancelStateDto {
    /// Calling it off now stops it.
    Open,
    /// Called off; the work is stopping.
    Requested,
    /// Past the point where it could stop: it runs to its end.
    Closed,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressStatusDto {
    Running,
    Done,
    /// `error` is the failure the initiator of the work got too, translated the same way.
    Failed {
        error: FrontendError,
    },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, Type, Event)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEventTypeDto {
    JavaDownload {
        version: u32,
    },
    PackFileDownload {
        instance_path: String,
        pack_name: String,
        icon: Option<String>,
        pack_version: String,
    },
    PackDownload {
        instance_path: String,
        pack_name: String,
        icon: Option<PathBuf>,
        pack_id: Option<String>,
        pack_version: Option<String>,
    },
    MinecraftDownload {
        instance_id: String,
        instance_name: String,
    },
    MinecraftVersionDownload {
        version: String,
    },
    MinecraftAssetsDownload {
        version: String,
        file_count: u32,
    },
    FileDownload {
        file_name: String,
    },
    InstanceUpdate {
        instance_id: String,
        instance_name: String,
    },
    ZipExtract {
        instance_path: String,
        instance_name: String,
    },
    LauncherUpdate {
        version: String,
        current_version: String,
        phase: LauncherUpdatePhaseDto,
    },
    PluginDownload {
        plugin_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum LauncherUpdatePhaseDto {
    Started,
    Progress,
    Finished,
    Error,
}

impl From<ProgressEvent> for ProgressEventDto {
    fn from(value: ProgressEvent) -> Self {
        Self {
            seq: 0,
            event: value.event.into(),
            progress_bar_id: value.progress_bar_id,
            fraction: value.fraction,
            message: value.message,
            stage: value.stage.map(Into::into),
            status: value.status.into(),
            parent_id: value.parent_id,
            mirror_of: value.mirror_of,
            bytes_done: bytes_to_number(value.bytes_done),
            bytes_total: bytes_to_number(value.bytes_total),
            speed: value.speed,
            cancel: value.cancel.map(Into::into),
        }
    }
}

impl From<CancelState> for CancelStateDto {
    fn from(value: CancelState) -> Self {
        match value {
            CancelState::Open => Self::Open,
            CancelState::Requested => Self::Requested,
            CancelState::Closed => Self::Closed,
        }
    }
}

impl From<ProgressStage> for ProgressStageDto {
    fn from(value: ProgressStage) -> Self {
        match value {
            ProgressStage::Preparing => Self::Preparing,
            ProgressStage::Downloading => Self::Downloading,
            ProgressStage::Verifying => Self::Verifying,
            ProgressStage::Installing => Self::Installing,
            ProgressStage::Extracting => Self::Extracting,
            ProgressStage::Finishing => Self::Finishing,
            ProgressStage::ByPlugin => Self::ByPlugin,
        }
    }
}

impl From<ProgressStatus> for ProgressStatusDto {
    fn from(value: ProgressStatus) -> Self {
        match value {
            ProgressStatus::Running => Self::Running,
            ProgressStatus::Done => Self::Done,
            ProgressStatus::Failed(failure) => Self::Failed {
                error: failure.error().into(),
            },
            ProgressStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<ProgressEventType> for ProgressEventTypeDto {
    fn from(value: ProgressEventType) -> Self {
        match value {
            ProgressEventType::JavaDownload { version } => Self::JavaDownload { version },
            ProgressEventType::PackFileDownload {
                instance_path,
                pack_name,
                icon,
                pack_version,
            } => Self::PackFileDownload {
                instance_path,
                pack_name,
                icon,
                pack_version,
            },
            ProgressEventType::PackDownload {
                instance_path,
                pack_name,
                icon,
                pack_id,
                pack_version,
            } => Self::PackDownload {
                instance_path,
                pack_name,
                icon,
                pack_id,
                pack_version,
            },
            ProgressEventType::MinecraftDownload {
                instance_id,
                instance_name,
            } => Self::MinecraftDownload {
                instance_id: instance_id.to_string(),
                instance_name,
            },
            ProgressEventType::MinecraftVersionDownload { version } => {
                Self::MinecraftVersionDownload { version }
            }
            ProgressEventType::MinecraftAssetsDownload {
                version,
                file_count,
            } => Self::MinecraftAssetsDownload {
                version,
                file_count,
            },
            ProgressEventType::FileDownload { file_name } => Self::FileDownload { file_name },
            ProgressEventType::InstanceUpdate {
                instance_id,
                instance_name,
            } => Self::InstanceUpdate {
                instance_id: instance_id.to_string(),
                instance_name,
            },
            ProgressEventType::ZipExtract {
                instance_path,
                instance_name,
            } => Self::ZipExtract {
                instance_path,
                instance_name,
            },
            ProgressEventType::PluginDownload { plugin_name } => {
                Self::PluginDownload { plugin_name }
            }
        }
    }
}

const UPDATE_PROGRESS_BAR_UUID: Uuid = Uuid::from_u128(0x1d1d98b5_eb55_4815_9349_8250b034aba3);

impl From<UpdateProgress> for ProgressEventDto {
    fn from(value: UpdateProgress) -> Self {
        let phase = (&value.phase).into();
        let status = match value.phase {
            UpdatePhase::Started | UpdatePhase::Progress => ProgressStatusDto::Running,
            UpdatePhase::Finished => ProgressStatusDto::Done,
            UpdatePhase::Error(error) => ProgressStatusDto::Failed {
                error: FrontendError::Update(error.into()),
            },
        };
        Self {
            seq: 0,
            event: ProgressEventTypeDto::LauncherUpdate {
                version: value.version,
                current_version: value.current_version,
                phase,
            },
            progress_bar_id: UPDATE_PROGRESS_BAR_UUID,
            fraction: value.fraction,
            message: "Launcher is updating".to_owned(),
            stage: None,
            status,
            parent_id: None,
            mirror_of: None,
            bytes_done: None,
            bytes_total: None,
            speed: None,
            cancel: None,
        }
    }
}

impl From<&UpdatePhase> for LauncherUpdatePhaseDto {
    fn from(value: &UpdatePhase) -> Self {
        match value {
            UpdatePhase::Started => Self::Started,
            UpdatePhase::Progress => Self::Progress,
            UpdatePhase::Finished => Self::Finished,
            UpdatePhase::Error(_) => Self::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::events::ProgressFailure;
    use aether_core::features::instance::InstanceError;
    use serde_json::json;

    use super::*;

    #[test]
    fn should_serialize_a_failed_status_with_the_error_the_initiator_gets() {
        let status = ProgressStatus::Failed(ProgressFailure::new(InstanceError::NotFound {
            instance_id: "abc".to_owned(),
        }));

        assert_eq!(
            serde_json::to_value(ProgressStatusDto::from(status)).unwrap(),
            json!({"type": "failed", "error": {
                "type": "instance",
                "payload": {"code": "NOT_FOUND", "payload": {"instance_id": "abc"}},
            }})
        );
    }

    #[test]
    fn should_serialize_a_failed_launcher_update_with_its_update_error() {
        let dto = ProgressEventDto::from(UpdateProgress {
            fraction: None,
            phase: UpdatePhase::Error(crate::features::update::UpdateError::NoPendingUpdate),
            version: "2.0.0".to_owned(),
            current_version: "1.0.0".to_owned(),
        });

        assert_eq!(
            serde_json::to_value(dto.status).unwrap(),
            json!({"type": "failed", "error": {"type": "update", "payload": {"code": "NO_PENDING_UPDATE"}}})
        );
    }
}
