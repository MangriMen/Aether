/// Checksum for download verification.
#[derive(Debug, Clone)]
pub struct Checksum {
    /// Hash algorithm name (e.g. "sha1", "sha512").
    pub algorithm: String,
    /// Hex-encoded hash value.
    pub value: String,
}

/// A declarative download instruction returned by a `ContentSource::get_version_info`
/// for `VersionPayload::Asset` variants.
///
/// The provider tells the host *what* to download and *where* to put it,
/// but does NOT perform the actual download. This avoids piping large
/// binary payloads through WASM plugin boundaries.
#[derive(Debug, Clone)]
pub struct DownloadInstruction {
    /// The URL to download the file from.
    pub url: String,
    /// The target file name.
    pub file_name: String,
    /// Optional checksum for integrity verification.
    pub checksum: Option<Checksum>,
    /// Optional extra HTTP headers for the download request.
    pub headers: Option<std::collections::HashMap<String, String>>,

    // ── Metadata for ContentFile construction ──
    pub content_path: std::path::PathBuf,
    pub content_type: super::ContentType,
    pub content_id: String,
    pub content_version: String,
    pub provider_id: super::ProviderId,
    pub name: Option<String>,
    pub size: u64,
}
