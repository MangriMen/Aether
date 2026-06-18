use std::path::PathBuf;

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ProgressEvent {
    pub event: ProgressEventType,
    pub progress_bar_id: Uuid,
    pub fraction: Option<f64>, // None means the loading is done
    pub message: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ProgressEventType {
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
    PluginDownload {
        plugin_name: String,
    },
}
