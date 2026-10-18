use std::path::PathBuf;

use aether_core::features::events::{
    ProgressEvent, ProgressEventType, ProgressStage, ProgressStatus,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;

use crate::features::update::{UpdatePhase, UpdateProgress};

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEventDto {
    pub event: ProgressEventTypeDto,
    pub progress_bar_id: Uuid,
    pub fraction: Option<f64>, // None means the loading is over; `status` tells how it ended
    pub message: String,
    pub stage: Option<ProgressStageDto>,
    pub status: ProgressStatusDto,
    pub parent_id: Option<Uuid>,
    /// A number, not `u64`: specta refuses to export `u64`, and a download never nears 2^53 bytes.
    pub bytes_done: Option<f64>,
    /// A number, not `u64`, like `bytes_done`.
    pub bytes_total: Option<f64>,
    /// Bytes per second.
    pub speed: Option<f64>,
}

/// The bytes as a number the frontend reads: see [`ProgressEventDto::bytes_done`].
#[allow(clippy::cast_precision_loss)]
pub(super) fn bytes_to_number(bytes: Option<u64>) -> Option<f64> {
    bytes.map(|bytes| bytes as f64)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStageDto {
    Downloading,
    Verifying,
    Installing,
    Extracting,
    Finishing,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressStatusDto {
    Running,
    Done,
    Failed { message: String },
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
            event: value.event.into(),
            progress_bar_id: value.progress_bar_id,
            fraction: value.fraction,
            message: value.message,
            stage: value.stage.map(Into::into),
            status: value.status.into(),
            parent_id: value.parent_id,
            bytes_done: bytes_to_number(value.bytes_done),
            bytes_total: bytes_to_number(value.bytes_total),
            speed: value.speed,
        }
    }
}

impl From<ProgressStage> for ProgressStageDto {
    fn from(value: ProgressStage) -> Self {
        match value {
            ProgressStage::Downloading => Self::Downloading,
            ProgressStage::Verifying => Self::Verifying,
            ProgressStage::Installing => Self::Installing,
            ProgressStage::Extracting => Self::Extracting,
            ProgressStage::Finishing => Self::Finishing,
        }
    }
}

impl From<ProgressStatus> for ProgressStatusDto {
    fn from(value: ProgressStatus) -> Self {
        match value {
            ProgressStatus::Running => Self::Running,
            ProgressStatus::Done => Self::Done,
            ProgressStatus::Failed { message } => Self::Failed { message },
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
        let status = match value.phase {
            UpdatePhase::Started | UpdatePhase::Progress => ProgressStatusDto::Running,
            UpdatePhase::Finished => ProgressStatusDto::Done,
            UpdatePhase::Error => ProgressStatusDto::Failed {
                message: "Launcher update failed".to_owned(),
            },
        };
        Self {
            event: ProgressEventTypeDto::LauncherUpdate {
                version: value.version,
                current_version: value.current_version,
                phase: value.phase.into(),
            },
            progress_bar_id: UPDATE_PROGRESS_BAR_UUID,
            fraction: value.fraction,
            message: "Launcher is updating".to_owned(),
            stage: None,
            status,
            parent_id: None,
            bytes_done: None,
            bytes_total: None,
            speed: None,
        }
    }
}

impl From<UpdatePhase> for LauncherUpdatePhaseDto {
    fn from(value: UpdatePhase) -> Self {
        match value {
            UpdatePhase::Started => Self::Started,
            UpdatePhase::Progress => Self::Progress,
            UpdatePhase::Finished => Self::Finished,
            UpdatePhase::Error => Self::Error,
        }
    }
}
