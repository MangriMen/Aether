#[derive(Debug)]
pub struct FsPluginStorageConstants {
    pub manifest_filename: &'static str,
    pub capabilities_filename: &'static str,
}

impl Default for FsPluginStorageConstants {
    fn default() -> Self {
        Self {
            manifest_filename: "manifest.json",
            capabilities_filename: "capabilities.json",
        }
    }
}
