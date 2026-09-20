#[derive(Debug)]
pub struct ZipPluginExtractorConstants {
    pub manifest_filename: &'static str,
    /// Upper bound on the number of entries in a plugin archive (zip-bomb guard).
    pub max_entries: usize,
    /// Upper bound on the total uncompressed size of a plugin archive (zip-bomb guard).
    pub max_total_uncompressed_bytes: u64,
}

impl Default for ZipPluginExtractorConstants {
    fn default() -> Self {
        Self {
            manifest_filename: "manifest.json",
            max_entries: 10_000,
            max_total_uncompressed_bytes: 512 * 1024 * 1024,
        }
    }
}
