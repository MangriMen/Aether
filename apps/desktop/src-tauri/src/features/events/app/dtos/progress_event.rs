use std::path::PathBuf;

use aether_core::features::events::{ProgressEvent, ProgressEventType};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "index.ts")]
pub struct ProgressEventDto {
    pub event: ProgressEventTypeDto,
    pub progress_bar_id: Uuid,
    pub fraction: Option<f64>, // None means the loading is done
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "index.ts")]
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
    CheckingForUpdates,
    LauncherUpdate {
        version: String,
        current_version: String,
    },
    PluginDownload {
        plugin_name: String,
    },
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
            ProgressEventType::CheckingForUpdates => Self::CheckingForUpdates,
            ProgressEventType::LauncherUpdate {
                version,
                current_version,
            } => Self::LauncherUpdate {
                version,
                current_version,
            },
            ProgressEventType::PluginDownload { plugin_name } => {
                Self::PluginDownload { plugin_name }
            }
        }
    }
}
