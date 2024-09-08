use std::error::Error;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AetherLauncherError {
    pub message: String,
}

impl From<Box<dyn Error + 'static>> for AetherLauncherError {
    fn from(value: Box<dyn Error + 'static>) -> Self {
        return AetherLauncherError {
            message: value.to_string(),
        };
    }
}

impl From<anyhow::Error> for AetherLauncherError {
    fn from(value: anyhow::Error) -> Self {
        AetherLauncherError {
            message: value.to_string(),
        }
    }
}
pub type AetherLauncherResult<T, E = AetherLauncherError> = Result<T, E>;
