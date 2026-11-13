use crate::{
    features::{
        instance::PluginId,
        plugins::{ManifestError, PluginSourceType},
    },
    shared::io::domain::IoError,
};

/// What went wrong talking to a plugin source, as far as the core knows it: a typed error when
/// there is one, our own text when there is not.
#[derive(thiserror::Error, Debug)]
pub enum ProviderFailure {
    #[error("{0}")]
    Request(#[from] reqwest::Error),

    /// The middleware error is an opaque `anyhow::Error`; the typed causes it wraps are in its chain.
    #[error("{0}")]
    Middleware(anyhow::Error),

    #[error("HTTP {status}")]
    Status { status: u16, url: String },

    #[error("{0}")]
    Other(String),
}

impl From<reqwest_middleware::Error> for ProviderFailure {
    fn from(value: reqwest_middleware::Error) -> Self {
        match value {
            reqwest_middleware::Error::Reqwest(err) => Self::Request(err),
            reqwest_middleware::Error::Middleware(err) => Self::Middleware(err),
        }
    }
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

    #[error("Failed to fetch plugin info from provider {source_type}: {cause}")]
    ProviderFetchError {
        source_type: PluginSourceType,
        // Not `source`: that name is `thiserror`'s own for the error source.
        cause: ProviderFailure,
    },

    #[error("No release assets found for plugin from provider {source_type}")]
    ProviderNoAssets { source_type: PluginSourceType },

    /// The archive downloaded for the install is not the one the preview was made from.
    #[error("The release's archive has changed since it was previewed")]
    ArchiveChangedSincePreview,

    #[error("Plugin \"{plugin_id}\" has no remote source configured")]
    NoRemoteSource { plugin_id: PluginId },

    #[error("Failed to download plugin from {url}: {cause}")]
    DownloadFailed { url: String, cause: ProviderFailure },

    // ── Rate limiting ──
    #[error("Provider {source_type} rate limit exceeded. Retry after {retry_after:?}")]
    ProviderRateLimited {
        source_type: PluginSourceType,
        retry_after: Option<u32>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_the_middleware_error_when_a_middleware_error_becomes_a_failure() {
        let failure = ProviderFailure::from(reqwest_middleware::Error::Middleware(
            anyhow::anyhow!("retry budget spent"),
        ));

        assert!(
            matches!(failure, ProviderFailure::Middleware(ref err) if err.to_string() == "retry budget spent"),
            "{failure:?}"
        );
    }

    #[test]
    fn should_keep_the_typed_error_when_a_reqwest_error_becomes_a_failure() {
        let err = reqwest::Client::new().get("not a url").build().unwrap_err();

        let failure = ProviderFailure::from(reqwest_middleware::Error::Reqwest(err));

        assert!(
            matches!(failure, ProviderFailure::Request(_)),
            "{failure:?}"
        );
    }
}
