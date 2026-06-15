#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Unsupported API version")]
    UnsupportedApi,

    #[error("Invalid path mapping")]
    InvalidPathMapping,
}
