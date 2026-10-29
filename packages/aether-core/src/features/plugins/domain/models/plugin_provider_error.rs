use crate::{
    features::{
        instance::PluginId,
        plugins::{ManifestError, PluginSourceType},
    },
    shared::io::domain::IoError,
};

/// Errors of obtaining a plugin from a source: remote provider, download, archive extraction.
#[derive(thiserror::Error, Debug)]
pub enum PluginProviderError {
    #[error("Plugin manifest not found: {path}")]
    ManifestNotFound { path: String },

    #[error("Invalid plugin manifest format: {error}")]
    InvalidManifestFormat { error: String },

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

    #[error("Failed to fetch plugin info from provider {source_type}: {details}")]
    ProviderFetchError {
        source_type: PluginSourceType,
        details: String,
    },

    #[error("No release assets found for plugin from provider {source_type}")]
    ProviderNoAssets { source_type: PluginSourceType },

    #[error("Plugin \"{plugin_id}\" has no remote source configured")]
    NoRemoteSource { plugin_id: PluginId },

    #[error("Failed to download plugin from {url}: {details}")]
    DownloadFailed { url: String, details: String },

    // ── Rate limiting ──
    #[error("Provider {source_type} rate limit exceeded. Retry after {retry_after:?}")]
    ProviderRateLimited {
        source_type: PluginSourceType,
        retry_after: Option<u32>,
    },
}
