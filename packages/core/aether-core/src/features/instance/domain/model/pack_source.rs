use super::ProviderId;

/// Describes where a pack comes from — the input for `PackManager::install`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PackSource {
    /// A pack from a content registry (Modrinth, `CurseForge`, etc.).
    Registry {
        provider_id: ProviderId,
        pack_id: String,
        version_id: String,
    },
    /// A local pack file (e.g., `.mrpack`, `.zip`, `pack.toml`).
    LocalFile(String),
    /// A remote pack URL.
    RemoteUrl(String),
}

/// Parameters passed to `PackManager::install`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PackInstallParams {
    /// How to resolve the pack content.
    pub source: PackSource,
    /// Optional override for the pack version.
    pub version: Option<String>,
}

/// Context available to `PackManager` implementations for downloads.
/// Provides access to authentication and HTTP clients as needed.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DownloadContext {
    // Future: auth registry, HTTP client config, etc.
}
