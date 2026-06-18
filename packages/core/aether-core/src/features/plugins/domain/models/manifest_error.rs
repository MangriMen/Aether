#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Unsupported API version")]
    UnsupportedApi,

    #[error("Invalid semver in {field}: \"{value}\" — {error}")]
    InvalidSemver {
        field: String,
        value: String,
        error: String,
    },

    #[error("Invalid path mapping")]
    InvalidPathMapping,
}
