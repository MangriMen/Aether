use std::path::PathBuf;

use uuid::Uuid;

use crate::features::instance::InstanceId;

#[derive(Clone, Debug)]
pub struct ProgressEvent {
    pub event: ProgressEventType,
    pub progress_bar_id: Uuid,
    /// Completion fraction; `None` means the loading is over (see `status` for how it ended).
    pub fraction: Option<f64>,
    /// Free-form text; the fallback for listeners that do not know `stage`.
    pub message: String,
    pub stage: Option<ProgressStage>,
    pub status: ProgressStatus,
    /// The bar this one is a part of, if any.
    pub parent_id: Option<Uuid>,
    pub bytes_done: Option<u64>,
    pub bytes_total: Option<u64>,
    /// Transfer speed in bytes per second.
    pub speed: Option<f64>,
}

/// What a progress bar is busy with right now.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ProgressStage {
    Downloading,
    Verifying,
    Installing,
    Extracting,
    Finishing,
}

/// Where a progress bar is in its life.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub enum ProgressStatus {
    #[default]
    Running,
    Done,
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ProgressEventType {
    JavaDownload {
        version: u32,
    },
    PackFileDownload {
        instance_path: String,
        pack_name: String,
        /// Icon as a string (not a local path), if any.
        icon: Option<String>,
        pack_version: String,
    },
    PackDownload {
        instance_path: String,
        pack_name: String,
        /// Local path of the icon, if any.
        icon: Option<PathBuf>,
        pack_id: Option<String>,
        pack_version: Option<String>,
    },
    MinecraftDownload {
        instance_id: InstanceId,
        instance_name: String,
    },
    InstanceUpdate {
        instance_id: InstanceId,
        instance_name: String,
    },
    ZipExtract {
        instance_path: String,
        instance_name: String,
    },
    PluginDownload {
        plugin_name: String,
    },
}
