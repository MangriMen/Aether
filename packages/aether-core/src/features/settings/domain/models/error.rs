use crate::features::instance::ProviderIdError;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Settings file not found. Please run initial setup.")]
    NotFound,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid content provider choice for platform '{platform}': {reason}")]
    InvalidPlatformProvider {
        platform: String,
        reason: ProviderIdError,
    },

    #[error("The number of concurrent downloads must be at least 1")]
    InvalidMaxConcurrentDownloads,

    #[error("The memory amount must be at least 1 MB")]
    InvalidMemory,
}
