use std::path::{Path, PathBuf};

use crate::features::instance::{InstanceId, PluginId};

pub const METADATA_FOLDER_NAME: &str = ".minecraft";
pub const CACHE_FOLDER_NAME: &str = "cache";
pub const INSTANCES_FOLDER_NAME: &str = "instances";
/// Subdirectory of an instance that holds state owned by the core, not by the pack:
/// `instance.json` and, under `plugins/<plugin_id>/`, each plugin's own per-instance state.
pub const INSTANCE_METADATA_FOLDER_NAME: &str = ".metadata";
pub const PLUGINS_FOLDER_NAME: &str = "plugins";
pub const PLUGIN_SETTINGS_FILE_NAME: &str = "settings.toml";

const INSTANCE_FILE_NAME: &str = "instance.json";

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

    /// Get the directory of the log configs the game versions name
    #[inline]
    pub fn log_configs_dir(&self) -> PathBuf {
        self.assets_dir().join("log_configs")
    }

    /// Where the log config `id` of a version goes. `None` for an id that is not a plain file
    /// name: it comes from the version's metadata and must not point outside the directory.
    pub fn log_config_path(&self, id: &str) -> Option<PathBuf> {
        let is_plain = !id.is_empty()
            && Path::new(id).file_name().is_some_and(|name| name == id)
            && !id.contains('\\');
        is_plain.then(|| self.log_configs_dir().join(id))
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

    /// Get the directory for a specific instance
    #[inline]
    pub fn instance_dir(&self, id: &InstanceId) -> PathBuf {
        self.instances_dir().join(id.as_str())
    }

    /// Where a removed instance directory waits to be deleted. Outside `instances_dir` so that a
    /// half-deleted leftover is never listed as an instance, yet on the same volume so moving a
    /// directory here is a rename. Under `temp_dir`, which is swept of interrupted work at startup.
    #[inline]
    pub fn removed_instances_dir(&self) -> PathBuf {
        self.temp_dir().join("removed-instances")
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
    pub fn plugin_dir(&self, id: &PluginId) -> PathBuf {
        self.plugins_dir().join(id.as_str())
    }

    #[inline]
    pub fn plugin_settings(&self, id: &PluginId) -> PathBuf {
        self.plugin_dir(id).join(PLUGIN_SETTINGS_FILE_NAME)
    }

    #[inline]
    pub fn plugin_cache_dir(&self, id: &PluginId) -> PathBuf {
        self.cache_dir().join("plugins").join(id.as_str())
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
    use std::path::{Path, PathBuf};

    use super::LocationInfo;
    use crate::features::instance::{InstanceId, PluginId};

    #[test]
    fn should_pin_every_path_accessor() {
        let li = LocationInfo::new(PathBuf::from("s"), PathBuf::from("c"));
        let id = InstanceId::try_from("i").unwrap();
        let pl = PluginId::from_static("pl");
        let p = |s: &str| PathBuf::from("c").join(s);
        assert_eq!(li.settings_dir(), Path::new("s"));
        assert_eq!(li.config_dir(), Path::new("c"));
        assert_eq!(li.metadata_dir(), p(".minecraft"));
        assert_eq!(li.data_dir(), p("data"));
        assert_eq!(li.db_path(), p("data").join("app.db"));
        let meta = p(".minecraft");
        assert_eq!(li.versions_dir(), meta.join("versions"));
        assert_eq!(li.version_dir("1.20"), meta.join("versions").join("1.20"));
        assert_eq!(li.libraries_dir(), meta.join("libraries"));
        assert_eq!(li.assets_dir(), meta.join("assets"));
        assert_eq!(li.assets_index_dir(), meta.join("assets").join("indexes"));
        assert_eq!(li.objects_dir(), meta.join("assets").join("objects"));
        assert_eq!(
            li.object_dir("abcdef"),
            meta.join("assets")
                .join("objects")
                .join("ab")
                .join("abcdef")
        );
        assert_eq!(li.legacy_assets_dir(), meta.join("resources"));
        assert_eq!(li.natives_dir(), meta.join("natives"));
        assert_eq!(
            li.version_natives_dir("1.20"),
            meta.join("natives").join("1.20")
        );
        let inst = p("instances");
        assert_eq!(li.instances_dir(), inst);
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
        let cache = p("cache");
        assert_eq!(li.cache_dir(), cache);
        assert_eq!(li.assets_cache_dir(), cache.join("assets"));
        assert_eq!(li.java_dir(), cache.join("java"));
        assert_eq!(li.plugins_dir(), p("plugins"));
        assert_eq!(li.plugin_dir(&pl), p("plugins").join("pl"));
        assert_eq!(
            li.plugin_settings(&pl),
            p("plugins").join("pl").join("settings.toml")
        );
        assert_eq!(li.plugin_cache_dir(&pl), cache.join("plugins").join("pl"));
        assert_eq!(
            li.instance_plugin_dir(&id, &pl),
            im.join("plugins").join("pl")
        );
        assert_eq!(
            li.crash_reports_dir(&id),
            inst.join("i").join("crash-reports")
        );
        assert_eq!(li.wasm_cache_config(), cache.join("wasm.toml"));
        assert_eq!(li.wasm_cache_dir(), cache.join("wasm"));
        assert_eq!(li.temp_dir(), cache.join("temp"));
        assert_eq!(
            li.removed_instances_dir(),
            cache.join("temp").join("removed-instances")
        );
    }
}
