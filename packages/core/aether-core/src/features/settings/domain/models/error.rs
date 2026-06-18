#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Settings file not found. Please run initial setup.")]
    NotFound,

    #[error("Storage error: {0}")]
    Storage(String),
}
