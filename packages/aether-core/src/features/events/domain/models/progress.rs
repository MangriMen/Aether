use std::path::PathBuf;

use uuid::Uuid;

/// A progress update for a long-running operation, identified by its progress bar.
#[derive(Clone, Debug)]
pub struct ProgressEvent {
    /// What operation this progress belongs to.
    pub event: ProgressEventType,
    /// Identifier of the progress bar the update targets.
    pub progress_bar_id: Uuid,
    /// Completion fraction; `None` means the loading is done.
    pub fraction: Option<f64>,
    /// Human-readable status message.
    pub message: String,
}

/// The kind of operation a [`ProgressEvent`] reports on, with its context.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ProgressEventType {
    /// Downloading a Java runtime.
    JavaDownload {
        /// Java major version being downloaded.
        version: u32,
    },
    /// Downloading a modpack file.
    PackFileDownload {
        /// Path of the target instance.
        instance_path: String,
        /// Modpack name.
        pack_name: String,
        /// Modpack icon, if any.
        icon: Option<String>,
        /// Modpack version.
        pack_version: String,
    },
    /// Downloading a modpack.
    PackDownload {
        /// Path of the target instance.
        instance_path: String,
        /// Modpack name.
        pack_name: String,
        /// Local path of the modpack icon, if any.
        icon: Option<PathBuf>,
        /// Modpack identifier, if known.
        pack_id: Option<String>,
        /// Modpack version, if known.
        pack_version: Option<String>,
    },
    /// Downloading Minecraft files for an instance.
    MinecraftDownload {
        /// Identifier of the instance.
        instance_id: String,
        /// Display name of the instance.
        instance_name: String,
    },
    /// Updating an instance.
    InstanceUpdate {
        /// Identifier of the instance.
        instance_id: String,
        /// Display name of the instance.
        instance_name: String,
    },
    /// Extracting an archive into an instance.
    ZipExtract {
        /// Path of the target instance.
        instance_path: String,
        /// Display name of the instance.
        instance_name: String,
    },
    /// Downloading a plugin.
    PluginDownload {
        /// Name of the plugin.
        plugin_name: String,
    },
}
