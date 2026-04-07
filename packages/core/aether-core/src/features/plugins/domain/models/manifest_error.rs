use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Unsupported API version")]
    UnsupportedApi,

    #[error("Invalid path mapping")]
    InvalidPathMapping,

    #[error("Invalid file path: {path:?}")]
    InvalidFilePath { path: PathBuf },
}
