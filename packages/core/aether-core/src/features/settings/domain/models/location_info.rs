use std::path::{Component, Path, PathBuf};

pub const METADATA_FOLDER_NAME: &str = ".minecraft";
pub const CACHE_FOLDER_NAME: &str = "cache";
pub const INSTANCES_FOLDER_NAME: &str = "instances";
/// Subdirectory of an instance that holds state owned by the core, not by the pack:
/// `instance.json` and, under `plugins/<plugin_id>/`, each plugin's own per-instance state.
pub const INSTANCE_METADATA_FOLDER_NAME: &str = ".metadata";
pub const PLUGINS_FOLDER_NAME: &str = "plugins";
pub const PLUGIN_SETTINGS_FILE_NAME: &str = "settings.toml";

#[derive(Debug)]
pub struct LocationInfo {
    settings_dir: PathBuf, // Base settings directory - app database
    /// Changeable through settings
    config_dir: PathBuf, // Config directory - instances, minecraft files, etc.
}

impl LocationInfo {
    pub fn new(settings_dir: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            settings_dir,
            config_dir,
        }
    }

    #[inline]
    pub fn settings_dir(&self) -> &Path {
        &self.settings_dir
    }

    #[inline]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get the Minecraft instance metadata directory
    #[inline]
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join(METADATA_FOLDER_NAME)
    }

    #[inline]
    pub fn data_dir(&self) -> PathBuf {
        self.config_dir.join("data")
    }

    #[inline]
    pub fn db_path(&self) -> PathBuf {
        self.data_dir().join("app.db")
    }

    /// Get the Minecraft versions metadata directory
    #[inline]
    pub fn versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("versions")
    }

    /// Get the metadata directory for a given version
    #[inline]
    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.versions_dir().join(version)
    }

    /// Get the Minecraft libraries metadata directory
    #[inline]
    pub fn libraries_dir(&self) -> PathBuf {
        self.metadata_dir().join("libraries")
    }

    /// Get the Minecraft assets metadata directory
    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("assets")
    }

    /// Get the assets index directory
    #[inline]
    pub fn assets_index_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    /// Get the assets objects directory
    #[inline]
    pub fn objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    /// Get the directory for a specific object
    #[inline]
    pub fn object_dir(&self, hash: &str) -> PathBuf {
        self.objects_dir().join(&hash[..2]).join(hash)
    }

    /// Get the Minecraft legacy assets metadata directory
    #[inline]
    pub fn legacy_assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("resources")
    }

    /// Get the Minecraft legacy assets metadata directory
    #[inline]
    pub fn natives_dir(&self) -> PathBuf {
        self.metadata_dir().join("natives")
    }

    /// Get the natives directory for a version of Minecraft
    #[inline]
    pub fn version_natives_dir(&self, version: &str) -> PathBuf {
        self.natives_dir().join(version)
    }

    /// Get the instances directory for created instances
    #[inline]
    pub fn instances_dir(&self) -> PathBuf {
        self.config_dir.join(INSTANCES_FOLDER_NAME)
    }

    /// Whether `id` can only name a directory directly under `instances/`.
    ///
    /// Instance ids also come from plugins. `..`, a separator or an absolute path would make
    /// [`Self::instance_dir`] point anywhere in the config directory, e.g. at another plugin's
    /// `verification.json`, so whatever takes an id from outside checks it here first.
    pub fn is_plain_instance_id(id: &str) -> bool {
        // `:` makes a drive-relative `C:x` or an alternate data stream on Windows.
        let forbidden: &[char] = if cfg!(windows) {
            &['/', '\\', '\0', ':']
        } else {
            &['/', '\\', '\0']
        };
        if id.is_empty() || id.contains(forbidden) {
            return false;
        }

        let mut components = Path::new(id).components();
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
    }

    /// Get the directory for a specific instance
    #[inline]
    pub fn instance_dir(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id)
    }

    /// Get the metadata directory for a specific instance
    #[inline]
    pub fn instance_metadata_dir(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join(INSTANCE_METADATA_FOLDER_NAME)
    }

    /// Get the metadata directory for a specific instance folder
    #[inline]
    pub fn instance_metadata_dir_with_instance_dir(&self, instance_dir: &Path) -> PathBuf {
        instance_dir.join(INSTANCE_METADATA_FOLDER_NAME)
    }

    /// Get the metadata file for a specific instance
    #[inline]
    pub fn instance_metadata_file(&self, id: &str) -> PathBuf {
        self.instance_metadata_dir(id).join("instance.json")
    }

    /// Get the metadata file for a specific instance folder
    #[inline]
    pub fn instance_metadata_file_with_instance_dir(&self, instance_dir: &Path) -> PathBuf {
        self.instance_metadata_dir_with_instance_dir(instance_dir)
            .join("instance.json")
    }

    /// Get the pack dir for a specific instance
    #[inline]
    pub fn instance_pack_dir(&self, id: &str) -> PathBuf {
        self.instance_metadata_dir(id).join("pack")
    }

    #[inline]
    pub fn instance_pack(&self, id: &str) -> PathBuf {
        self.instance_pack_dir(id).join("content.toml")
    }

    /// Get the cache directory
    #[inline]
    pub fn cache_dir(&self) -> PathBuf {
        self.config_dir.join(CACHE_FOLDER_NAME)
    }

    /// Get the cache directory
    #[inline]
    pub fn assets_cache_dir(&self) -> PathBuf {
        self.cache_dir().join("assets")
    }

    /// Get the Minecraft java versions metadata directory
    #[inline]
    pub fn java_dir(&self) -> PathBuf {
        self.cache_dir().join("java")
    }

    /// Get the plugins directory
    #[inline]
    pub fn plugins_dir(&self) -> PathBuf {
        self.config_dir.join(PLUGINS_FOLDER_NAME)
    }

    /// Get the directory for a specific plugin
    #[inline]
    pub fn plugin_dir(&self, id: &str) -> PathBuf {
        self.plugins_dir().join(id)
    }

    #[inline]
    pub fn plugin_settings(&self, id: &str) -> PathBuf {
        self.plugin_dir(id).join(PLUGIN_SETTINGS_FILE_NAME)
    }

    #[inline]
    pub fn plugin_cache_dir(&self, id: &str) -> PathBuf {
        self.cache_dir().join("plugins").join(id)
    }

    /// Get the directory for a specific plugin inside an instance
    #[inline]
    pub fn instance_plugin_dir(&self, id: &str, plugin_id: &str) -> PathBuf {
        self.instance_metadata_dir(id)
            .join(PLUGINS_FOLDER_NAME)
            .join(plugin_id)
    }

    #[inline]
    pub fn crash_reports_dir(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join("crash-reports")
    }

    #[inline]
    pub fn wasm_cache_config(&self) -> PathBuf {
        self.cache_dir().join("wasm.toml")
    }

    #[inline]
    pub fn wasm_cache_dir(&self) -> PathBuf {
        self.cache_dir().join("wasm")
    }

    #[inline]
    pub fn temp_dir(&self) -> PathBuf {
        self.cache_dir().join("temp")
    }
}

#[cfg(test)]
mod tests {
    use super::LocationInfo;

    #[test]
    fn should_accept_an_instance_id_when_it_is_a_single_name() {
        for id in ["inst-1", "My Pack", "pack.v2", ".hidden", "Сборка"] {
            assert!(LocationInfo::is_plain_instance_id(id), "{id}");
        }
    }

    #[test]
    fn should_reject_an_instance_id_when_it_leaves_the_instances_directory() {
        for id in [
            "",
            ".",
            "..",
            "../plugins/x",
            "..\\plugins\\x",
            "a/b",
            "a\\b",
            "/etc",
            "C:\\Windows",
            "\\\\server\\share",
            "a\0b",
        ] {
            assert!(!LocationInfo::is_plain_instance_id(id), "{id:?}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn should_reject_an_instance_id_when_it_has_a_drive_or_stream_on_windows() {
        for id in ["C:", "C:x", "pack:stream"] {
            assert!(!LocationInfo::is_plain_instance_id(id), "{id:?}");
        }
    }
}
