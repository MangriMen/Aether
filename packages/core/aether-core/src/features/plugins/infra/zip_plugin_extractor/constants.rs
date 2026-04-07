#[derive(Debug)]
pub struct ZipPluginExtractorConstants {
    pub manifest_filename: &'static str,
}

impl Default for ZipPluginExtractorConstants {
    fn default() -> Self {
        Self {
            manifest_filename: "manifest.json",
        }
    }
}
