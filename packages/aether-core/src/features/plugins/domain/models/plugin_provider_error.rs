use crate::{
    features::{
        instance::PluginId,
        plugins::{ManifestError, PluginSourceType},
    },
    shared::{error::BoxedSource, io::domain::IoError},
};

/// What went wrong talking to a plugin source. The client's errors are kept as the source; the
/// conversion from them lives with the client.
#[derive(thiserror::Error, Debug)]
pub enum ProviderFailure {
    /// The request did not get an answer, or the answer could not be read.
    #[error("{0}")]
    Request(#[source] BoxedSource),

    #[error("HTTP {status}")]
    Status { status: u16, url: String },

    #[error("the asset is larger than {limit} bytes")]
    TooLarge { limit: u64 },

    /// Only the source itself may serve plugin assets, over https.
    #[error("the asset URL is not one the provider serves over https")]
    UrlNotAllowed,

    #[error("the source has no releases")]
    NoReleases,

    #[error("\"{url}\" is not an address of a plugin on this source")]
    InvalidUrl { url: String },

    #[error("no provider is registered for this source")]
    NotRegistered,
}

/// Why a file of a plugin archive that must be JSON is not.
#[derive(thiserror::Error, Debug)]
pub enum ManifestFormatError {
    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("{name} is larger than {limit} bytes")]
    TooLarge { name: String, limit: u64 },
}

/// Errors of obtaining a plugin from a source: remote provider, download, archive extraction.
#[derive(thiserror::Error, Debug)]
pub enum PluginProviderError {
    #[error("Plugin manifest not found: {path}")]
    ManifestNotFound { path: String },

    #[error("Invalid plugin manifest format: {error}")]
    InvalidManifestFormat {
        #[source]
        error: ManifestFormatError,
    },

    #[error("Failed to extract plugin from: {from}")]
    ExtractionFailed { from: String },

    #[error("Invalid extraction format")]
    InvalidExtractionFormat,

    #[error("Failed to extract plugin files: {from}")]
    FileExtractionFailed { from: String },

    #[error("{0}")]
    Manifest(#[from] ManifestError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),

    #[error("Failed to fetch plugin info from provider {source_type}: {reason}")]
    ProviderFetchError {
        source_type: PluginSourceType,
        reason: ProviderFailure,
    },

    #[error("No release assets found for plugin from provider {source_type}")]
    ProviderNoAssets { source_type: PluginSourceType },

    /// The archive downloaded for the install is not the one the preview was made from.
    #[error("The release's archive has changed since it was previewed")]
    ArchiveChangedSincePreview,

    #[error("Plugin \"{plugin_id}\" has no remote source configured")]
    NoRemoteSource { plugin_id: PluginId },

    #[error("Failed to download plugin from {url}: {reason}")]
    DownloadFailed {
        url: String,
        reason: ProviderFailure,
    },

    // ── Rate limiting ──
    #[error("Provider {source_type} rate limit exceeded. Retry after {retry_after:?}")]
    ProviderRateLimited {
        source_type: PluginSourceType,
        retry_after: Option<u32>,
    },
}
