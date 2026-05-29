use std::path::{Path, PathBuf};

pub const METADATA_FOLDER_NAME: &str = ".minecraft";
pub const CACHE_FOLDER_NAME: &str = "cache";
pub const INSTANCES_FOLDER_NAME: &str = "instances";
pub const PLUGINS_FOLDER_NAME: &str = "plugins";

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

    /// Get the directory for a specific instance
    #[inline]
    pub fn instance_dir(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id)
    }

    /// Get the metadata directory for a specific instance
    #[inline]
    pub fn instance_metadata_dir(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join(".metadata")
    }

    /// Get the metadata directory for a specific instance folder
    #[inline]
    pub fn instance_metadata_dir_with_instance_dir(&self, instance_dir: &Path) -> PathBuf {
        instance_dir.join(".metadata")
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
        self.plugin_dir(id).join("settings.toml")
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
    use super::*;

    fn test_location_info() -> LocationInfo {
        LocationInfo::new(
            PathBuf::from("/aether/settings"),
            PathBuf::from("/aether/config"),
        )
    }

    #[test]
    fn should_return_settings_dir() {
        let info = test_location_info();

        assert_eq!(info.settings_dir(), Path::new("/aether/settings"));
    }

    #[test]
    fn should_return_config_dir() {
        let info = test_location_info();

        assert_eq!(info.config_dir(), Path::new("/aether/config"));
    }

    #[test]
    fn should_compute_metadata_dir() {
        let info = test_location_info();

        assert_eq!(info.metadata_dir(), PathBuf::from("/aether/config/.minecraft"));
    }

    #[test]
    fn should_compute_data_dir() {
        let info = test_location_info();

        assert_eq!(info.data_dir(), PathBuf::from("/aether/config/data"));
    }

    #[test]
    fn should_compute_db_path() {
        let info = test_location_info();

        assert_eq!(info.db_path(), PathBuf::from("/aether/config/data/app.db"));
    }

    #[test]
    fn should_compute_versions_dir() {
        let info = test_location_info();

        assert_eq!(
            info.versions_dir(),
            PathBuf::from("/aether/config/.minecraft/versions")
        );
    }

    #[test]
    fn should_compute_version_dir() {
        let info = test_location_info();

        assert_eq!(
            info.version_dir("1.20"),
            PathBuf::from("/aether/config/.minecraft/versions/1.20")
        );
    }

    #[test]
    fn should_compute_libraries_dir() {
        let info = test_location_info();

        assert_eq!(
            info.libraries_dir(),
            PathBuf::from("/aether/config/.minecraft/libraries")
        );
    }

    #[test]
    fn should_compute_assets_dir() {
        let info = test_location_info();

        assert_eq!(
            info.assets_dir(),
            PathBuf::from("/aether/config/.minecraft/assets")
        );
    }

    #[test]
    fn should_compute_assets_index_dir() {
        let info = test_location_info();

        assert_eq!(
            info.assets_index_dir(),
            PathBuf::from("/aether/config/.minecraft/assets/indexes")
        );
    }

    #[test]
    fn should_compute_objects_dir() {
        let info = test_location_info();

        assert_eq!(
            info.objects_dir(),
            PathBuf::from("/aether/config/.minecraft/assets/objects")
        );
    }

    #[test]
    fn should_compute_object_dir_from_hash() {
        let info = test_location_info();

        assert_eq!(
            info.object_dir("abc123xyz"),
            PathBuf::from("/aether/config/.minecraft/assets/objects/ab/abc123xyz")
        );
    }

    #[test]
    fn should_compute_legacy_assets_dir() {
        let info = test_location_info();

        assert_eq!(
            info.legacy_assets_dir(),
            PathBuf::from("/aether/config/.minecraft/resources")
        );
    }

    #[test]
    fn should_compute_natives_dir() {
        let info = test_location_info();

        assert_eq!(
            info.natives_dir(),
            PathBuf::from("/aether/config/.minecraft/natives")
        );
    }

    #[test]
    fn should_compute_version_natives_dir() {
        let info = test_location_info();

        assert_eq!(
            info.version_natives_dir("1.20"),
            PathBuf::from("/aether/config/.minecraft/natives/1.20")
        );
    }

    #[test]
    fn should_compute_instances_dir() {
        let info = test_location_info();

        assert_eq!(
            info.instances_dir(),
            PathBuf::from("/aether/config/instances")
        );
    }

    #[test]
    fn should_compute_instance_dir() {
        let info = test_location_info();

        assert_eq!(
            info.instance_dir("my-instance"),
            PathBuf::from("/aether/config/instances/my-instance")
        );
    }

    #[test]
    fn should_compute_instance_metadata_dir() {
        let info = test_location_info();

        assert_eq!(
            info.instance_metadata_dir("inst-1"),
            PathBuf::from("/aether/config/instances/inst-1/.metadata")
        );
    }

    #[test]
    fn should_compute_instance_metadata_file() {
        let info = test_location_info();

        assert_eq!(
            info.instance_metadata_file("inst-1"),
            PathBuf::from("/aether/config/instances/inst-1/.metadata/instance.json")
        );
    }

    #[test]
    fn should_compute_instance_metadata_file_with_instance_dir() {
        let info = test_location_info();

        assert_eq!(
            info.instance_metadata_file_with_instance_dir(Path::new("/custom/inst")),
            PathBuf::from("/custom/inst/.metadata/instance.json")
        );
    }

    #[test]
    fn should_compute_instance_pack_dir() {
        let info = test_location_info();

        assert_eq!(
            info.instance_pack_dir("inst-1"),
            PathBuf::from("/aether/config/instances/inst-1/.metadata/pack")
        );
    }

    #[test]
    fn should_compute_instance_pack() {
        let info = test_location_info();

        assert_eq!(
            info.instance_pack("inst-1"),
            PathBuf::from("/aether/config/instances/inst-1/.metadata/pack/content.toml")
        );
    }

    #[test]
    fn should_compute_cache_dir() {
        let info = test_location_info();

        assert_eq!(
            info.cache_dir(),
            PathBuf::from("/aether/config/cache")
        );
    }

    #[test]
    fn should_compute_assets_cache_dir() {
        let info = test_location_info();

        assert_eq!(
            info.assets_cache_dir(),
            PathBuf::from("/aether/config/cache/assets")
        );
    }

    #[test]
    fn should_compute_java_dir() {
        let info = test_location_info();

        assert_eq!(
            info.java_dir(),
            PathBuf::from("/aether/config/cache/java")
        );
    }

    #[test]
    fn should_compute_plugins_dir() {
        let info = test_location_info();

        assert_eq!(
            info.plugins_dir(),
            PathBuf::from("/aether/config/plugins")
        );
    }

    #[test]
    fn should_compute_plugin_dir() {
        let info = test_location_info();

        assert_eq!(
            info.plugin_dir("my-plugin"),
            PathBuf::from("/aether/config/plugins/my-plugin")
        );
    }

    #[test]
    fn should_compute_plugin_settings() {
        let info = test_location_info();

        assert_eq!(
            info.plugin_settings("my-plugin"),
            PathBuf::from("/aether/config/plugins/my-plugin/settings.toml")
        );
    }

    #[test]
    fn should_compute_plugin_cache_dir() {
        let info = test_location_info();

        assert_eq!(
            info.plugin_cache_dir("p1"),
            PathBuf::from("/aether/config/cache/plugins/p1")
        );
    }

    #[test]
    fn should_compute_instance_plugin_dir() {
        let info = test_location_info();

        assert_eq!(
            info.instance_plugin_dir("inst-1", "p1"),
            PathBuf::from("/aether/config/instances/inst-1/.metadata/plugins/p1")
        );
    }

    #[test]
    fn should_compute_crash_reports_dir() {
        let info = test_location_info();

        assert_eq!(
            info.crash_reports_dir("inst-1"),
            PathBuf::from("/aether/config/instances/inst-1/crash-reports")
        );
    }

    #[test]
    fn should_compute_temp_dir() {
        let info = test_location_info();

        assert_eq!(
            info.temp_dir(),
            PathBuf::from("/aether/config/cache/temp")
        );
    }
}
