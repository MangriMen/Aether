use crate::core::WindowLabel;

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Window with label '{}' already exists", label.as_ref())]
    AlreadyExists { label: WindowLabel },

    #[error("Window with label '{}' was not found", label.as_ref())]
    NotFound { label: WindowLabel },

    #[error("Tauri internal error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Platform error: {0}")]
    PlatformNotSupported(String),

    #[error("Unexpected error: {0}")]
    Other(String),
}
