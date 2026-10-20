#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Settings file not found. Please run initial setup.")]
    NotFound,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid content provider choice for platform '{platform}': {reason}")]
    InvalidPlatformProvider { platform: String, reason: String },

    #[error("The number of concurrent downloads must be at least 1")]
    InvalidMaxConcurrentDownloads,
}
