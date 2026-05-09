use std::path::PathBuf;

use aether_core::features::events::{ProgressEvent, ProgressEventType};
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
    pub fraction: Option<f64>, // None means the loading is done
    pub message: String,
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
                instance_id,
                instance_name,
            },
            ProgressEventType::InstanceUpdate {
                instance_id,
                instance_name,
            } => Self::InstanceUpdate {
                instance_id,
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
        Self {
            event: ProgressEventTypeDto::LauncherUpdate {
                version: value.version,
                current_version: value.current_version,
                phase: value.phase.into(),
            },
            progress_bar_id: UPDATE_PROGRESS_BAR_UUID,
            fraction: value.fraction,
            message: "Launcher is updating".to_owned(),
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
