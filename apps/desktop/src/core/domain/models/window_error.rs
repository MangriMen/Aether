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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_error_already_exists_format() {
        let err = WindowError::AlreadyExists {
            label: WindowLabel::Main,
        };
        assert_eq!(err.to_string(), "Window with label 'main' already exists");
    }

    #[test]
    fn window_error_not_found_format() {
        let err = WindowError::NotFound {
            label: WindowLabel::Main,
        };
        assert_eq!(err.to_string(), "Window with label 'main' was not found");
    }

    #[test]
    fn window_error_platform_not_supported_format() {
        let err = WindowError::PlatformNotSupported("linux".to_string());
        assert_eq!(err.to_string(), "Platform error: linux");
    }

    #[test]
    fn window_error_other_format() {
        let err = WindowError::Other("something went wrong".to_string());
        assert_eq!(err.to_string(), "Unexpected error: something went wrong");
    }
}
