//! The paths inside an instance's directory, on top of the settings' [`LocationInfo`].

use std::path::{Path, PathBuf};

use super::{INSTANCE_METADATA_FOLDER_NAME, InstanceId};
use crate::features::settings::{LocationInfo, PLUGINS_FOLDER_NAME};
use crate::shared::capability::PluginId;

const INSTANCE_FILE_NAME: &str = "instance.json";

impl LocationInfo {
    /// Get the directory for a specific instance
    #[inline]
    pub fn instance_dir(&self, id: &InstanceId) -> PathBuf {
        self.instances_dir().join(id.as_str())
    }

    /// Get the metadata directory for a specific instance
    #[inline]
    pub fn instance_metadata_dir(&self, id: &InstanceId) -> PathBuf {
        self.instance_dir(id).join(INSTANCE_METADATA_FOLDER_NAME)
    }

    /// Get the metadata directory for a specific instance folder
    #[inline]
    pub fn instance_metadata_dir_with_instance_dir(&self, instance_dir: &Path) -> PathBuf {
        instance_dir.join(INSTANCE_METADATA_FOLDER_NAME)
    }

    /// Get the metadata file for a specific instance
    #[inline]
    pub fn instance_metadata_file(&self, id: &InstanceId) -> PathBuf {
        self.instance_metadata_dir(id).join(INSTANCE_FILE_NAME)
    }

    /// Get the metadata file for a specific instance folder
    #[inline]
    pub fn instance_metadata_file_with_instance_dir(&self, instance_dir: &Path) -> PathBuf {
        self.instance_metadata_dir_with_instance_dir(instance_dir)
            .join(INSTANCE_FILE_NAME)
    }

    /// Get the pack dir for a specific instance
    #[inline]
    pub fn instance_pack_dir(&self, id: &InstanceId) -> PathBuf {
        self.instance_metadata_dir(id).join("pack")
    }

    #[inline]
    pub fn instance_pack(&self, id: &InstanceId) -> PathBuf {
        self.instance_pack_dir(id).join("content.toml")
    }

    /// Get the directory for a specific plugin inside an instance
    #[inline]
    pub fn instance_plugin_dir(&self, id: &InstanceId, plugin_id: &PluginId) -> PathBuf {
        self.instance_metadata_dir(id)
            .join(PLUGINS_FOLDER_NAME)
            .join(plugin_id.as_str())
    }

    #[inline]
    pub fn crash_reports_dir(&self, id: &InstanceId) -> PathBuf {
        self.instance_dir(id).join("crash-reports")
    }

    /// The game's own logs (`latest.log`, rotated `*.log.gz`).
    #[inline]
    pub fn logs_dir(&self, id: &InstanceId) -> PathBuf {
        self.instance_dir(id).join("logs")
    }

    /// Per-run captures of the game's console output, kept outside the game's own `logs`.
    #[inline]
    pub fn game_logs_dir(&self, id: &InstanceId) -> PathBuf {
        self.instance_metadata_dir(id).join("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_pin_every_instance_path_accessor() {
        let li = LocationInfo::new(PathBuf::from("s"), PathBuf::from("c"));
        let id = InstanceId::try_from("i").unwrap();
        let pl = PluginId::from_static("pl");
        let inst = PathBuf::from("c").join("instances");
        assert_eq!(li.instance_dir(&id), inst.join("i"));
        let im = inst.join("i").join(".metadata");
        assert_eq!(li.instance_metadata_dir(&id), im);
        assert_eq!(
            li.instance_metadata_dir_with_instance_dir(Path::new("x")),
            Path::new("x").join(".metadata")
        );
        assert_eq!(li.instance_metadata_file(&id), im.join("instance.json"));
        assert_eq!(
            li.instance_metadata_file_with_instance_dir(Path::new("x")),
            Path::new("x").join(".metadata").join("instance.json")
        );
        assert_eq!(li.instance_pack_dir(&id), im.join("pack"));
        assert_eq!(li.instance_pack(&id), im.join("pack").join("content.toml"));
        assert_eq!(
            li.instance_plugin_dir(&id, &pl),
            im.join("plugins").join("pl")
        );
        assert_eq!(
            li.crash_reports_dir(&id),
            inst.join("i").join("crash-reports")
        );
        assert_eq!(li.logs_dir(&id), inst.join("i").join("logs"));
        assert_eq!(li.game_logs_dir(&id), im.join("logs"));
    }
}
