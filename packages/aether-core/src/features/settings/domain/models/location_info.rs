use std::path::{Component, Path, PathBuf};

/// Game files shared by all instances (versions, libraries, assets), under the config directory.
pub const METADATA_FOLDER_NAME: &str = ".minecraft";
/// Downloaded and generated files that can be recreated, under the config directory.
pub const CACHE_FOLDER_NAME: &str = "cache";
/// One subdirectory per instance, under the config directory.
pub const INSTANCES_FOLDER_NAME: &str = "instances";
/// Subdirectory of an instance that holds state owned by the core, not by the pack:
/// `instance.json` and, under `plugins/<plugin_id>/`, each plugin's own per-instance state.
pub const INSTANCE_METADATA_FOLDER_NAME: &str = ".metadata";
pub const PLUGINS_FOLDER_NAME: &str = "plugins";
pub const PLUGIN_SETTINGS_FILE_NAME: &str = "settings.toml";

const INSTANCE_FILE_NAME: &str = "instance.json";

/// Appends every segment of `parts` to `base`.
fn extend<const N: usize>(base: impl AsRef<Path>, parts: [&str; N]) -> PathBuf {
    let mut path = base.as_ref().to_path_buf();
    path.extend(parts);
    path
}

/// Resolves every location the launcher uses on disk.
#[derive(Debug)]
pub struct LocationInfo {
    /// Where the application keeps its own settings.
    settings_dir: PathBuf,
    /// Root of everything else (instances, game files, caches); the user can move it.
    config_dir: PathBuf,
}

impl LocationInfo {
    pub fn new(settings_dir: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            settings_dir,
            config_dir,
        }
    }

    // --- roots and application data ---

    pub fn settings_dir(&self) -> &Path {
        &self.settings_dir
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn data_dir(&self) -> PathBuf {
        extend(&self.config_dir, ["data"])
    }

    pub fn db_path(&self) -> PathBuf {
        extend(self.data_dir(), ["app.db"])
    }

    // --- shared game files ---

    /// Root of the shared game files.
    pub fn metadata_dir(&self) -> PathBuf {
        extend(&self.config_dir, [METADATA_FOLDER_NAME])
    }

    /// Every installed game version lives in its own folder here.
    pub fn versions_dir(&self) -> PathBuf {
        extend(self.metadata_dir(), ["versions"])
    }

    /// Files of one game version.
    pub fn version_dir(&self, version: &str) -> PathBuf {
        extend(self.versions_dir(), [version])
    }

    pub fn libraries_dir(&self) -> PathBuf {
        extend(self.metadata_dir(), ["libraries"])
    }

    pub fn assets_dir(&self) -> PathBuf {
        extend(self.metadata_dir(), ["assets"])
    }

    /// Asset index files.
    pub fn assets_index_dir(&self) -> PathBuf {
        extend(self.assets_dir(), ["indexes"])
    }

    /// Content-addressed asset files.
    pub fn objects_dir(&self) -> PathBuf {
        extend(self.assets_dir(), ["objects"])
    }

    /// Location of the asset with the given hash; objects are sharded by the first two
    /// characters of the hash.
    pub fn object_dir(&self, hash: &str) -> PathBuf {
        extend(self.objects_dir(), [&hash[..2], hash])
    }

    /// Assets of old game versions that are not stored by hash.
    pub fn legacy_assets_dir(&self) -> PathBuf {
        extend(self.metadata_dir(), ["resources"])
    }

    /// Extracted native libraries, one folder per game version.
    pub fn natives_dir(&self) -> PathBuf {
        extend(self.metadata_dir(), ["natives"])
    }

    pub fn version_natives_dir(&self, version: &str) -> PathBuf {
        extend(self.natives_dir(), [version])
    }

    // --- instances ---

    pub fn instances_dir(&self) -> PathBuf {
        extend(&self.config_dir, [INSTANCES_FOLDER_NAME])
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

    pub fn instance_dir(&self, id: &str) -> PathBuf {
        extend(self.instances_dir(), [id])
    }

    /// Core-owned state of an instance.
    pub fn instance_metadata_dir(&self, id: &str) -> PathBuf {
        self.instance_metadata_dir_with_instance_dir(&self.instance_dir(id))
    }

    /// Same as [`Self::instance_metadata_dir`] for an already known instance directory.
    pub fn instance_metadata_dir_with_instance_dir(&self, instance_dir: &Path) -> PathBuf {
        extend(instance_dir, [INSTANCE_METADATA_FOLDER_NAME])
    }

    pub fn instance_metadata_file(&self, id: &str) -> PathBuf {
        extend(self.instance_metadata_dir(id), [INSTANCE_FILE_NAME])
    }

    /// Same as [`Self::instance_metadata_file`] for an already known instance directory.
    pub fn instance_metadata_file_with_instance_dir(&self, instance_dir: &Path) -> PathBuf {
        extend(
            self.instance_metadata_dir_with_instance_dir(instance_dir),
            [INSTANCE_FILE_NAME],
        )
    }

    pub fn instance_pack_dir(&self, id: &str) -> PathBuf {
        extend(self.instance_metadata_dir(id), ["pack"])
    }

    pub fn instance_pack(&self, id: &str) -> PathBuf {
        extend(self.instance_pack_dir(id), ["content.toml"])
    }

    pub fn crash_reports_dir(&self, id: &str) -> PathBuf {
        extend(self.instance_dir(id), ["crash-reports"])
    }

    // --- plugins ---

    pub fn plugins_dir(&self) -> PathBuf {
        extend(&self.config_dir, [PLUGINS_FOLDER_NAME])
    }

    pub fn plugin_dir(&self, id: &str) -> PathBuf {
        extend(self.plugins_dir(), [id])
    }

    pub fn plugin_settings(&self, id: &str) -> PathBuf {
        extend(self.plugin_dir(id), [PLUGIN_SETTINGS_FILE_NAME])
    }

    /// State a plugin keeps inside one instance.
    pub fn instance_plugin_dir(&self, id: &str, plugin_id: &str) -> PathBuf {
        extend(
            self.instance_metadata_dir(id),
            [PLUGINS_FOLDER_NAME, plugin_id],
        )
    }

    // --- cache ---

    pub fn cache_dir(&self) -> PathBuf {
        extend(&self.config_dir, [CACHE_FOLDER_NAME])
    }

    pub fn assets_cache_dir(&self) -> PathBuf {
        extend(self.cache_dir(), ["assets"])
    }

    /// Downloaded Java runtimes.
    pub fn java_dir(&self) -> PathBuf {
        extend(self.cache_dir(), ["java"])
    }

    pub fn plugin_cache_dir(&self, id: &str) -> PathBuf {
        extend(self.cache_dir(), ["plugins", id])
    }

    pub fn wasm_cache_config(&self) -> PathBuf {
        extend(self.cache_dir(), ["wasm.toml"])
    }

    pub fn wasm_cache_dir(&self) -> PathBuf {
        extend(self.cache_dir(), ["wasm"])
    }

    pub fn temp_dir(&self) -> PathBuf {
        extend(self.cache_dir(), ["temp"])
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

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

    #[test]
    fn should_pin_every_path_accessor() {
        let li = LocationInfo::new(PathBuf::from("s"), PathBuf::from("c"));
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
        assert_eq!(li.instance_dir("i"), inst.join("i"));
        let im = inst.join("i").join(".metadata");
        assert_eq!(li.instance_metadata_dir("i"), im);
        assert_eq!(
            li.instance_metadata_dir_with_instance_dir(Path::new("x")),
            Path::new("x").join(".metadata")
        );
        assert_eq!(li.instance_metadata_file("i"), im.join("instance.json"));
        assert_eq!(
            li.instance_metadata_file_with_instance_dir(Path::new("x")),
            Path::new("x").join(".metadata").join("instance.json")
        );
        assert_eq!(li.instance_pack_dir("i"), im.join("pack"));
        assert_eq!(li.instance_pack("i"), im.join("pack").join("content.toml"));
        let cache = p("cache");
        assert_eq!(li.cache_dir(), cache);
        assert_eq!(li.assets_cache_dir(), cache.join("assets"));
        assert_eq!(li.java_dir(), cache.join("java"));
        assert_eq!(li.plugins_dir(), p("plugins"));
        assert_eq!(li.plugin_dir("pl"), p("plugins").join("pl"));
        assert_eq!(
            li.plugin_settings("pl"),
            p("plugins").join("pl").join("settings.toml")
        );
        assert_eq!(li.plugin_cache_dir("pl"), cache.join("plugins").join("pl"));
        assert_eq!(
            li.instance_plugin_dir("i", "pl"),
            im.join("plugins").join("pl")
        );
        assert_eq!(
            li.crash_reports_dir("i"),
            inst.join("i").join("crash-reports")
        );
        assert_eq!(li.wasm_cache_config(), cache.join("wasm.toml"));
        assert_eq!(li.wasm_cache_dir(), cache.join("wasm"));
        assert_eq!(li.temp_dir(), cache.join("temp"));
    }
}
