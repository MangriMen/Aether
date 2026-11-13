use serde::Serialize;
use specta::Type;

use crate::core::{
    AppSettingsErrorDto, ApplicationErrorDto, AuthErrorDto, CauseDto, EventErrorDto,
    FileWatcherErrorDto, GameLogErrorDto, InstanceErrorDto, JavaErrorDto, MinecraftErrorDto,
    PluginErrorDto, ProcessErrorDto, RequestErrorDto, SettingsErrorDto, WindowErrorDto,
};

pub type FrontendResult<T, E = FrontendError> = Result<T, E>;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum FrontendError {
    AppSettings(AppSettingsErrorDto),

    Window(WindowErrorDto),

    Application(ApplicationErrorDto),

    Auth(AuthErrorDto),

    Event(EventErrorDto),

    FileWatcher(FileWatcherErrorDto),

    GameLog(GameLogErrorDto),

    Instance(InstanceErrorDto),

    Java(JavaErrorDto),

    Minecraft(MinecraftErrorDto),

    Plugin(PluginErrorDto),

    Process(ProcessErrorDto),

    Request(RequestErrorDto),

    Settings(SettingsErrorDto),
}

impl From<crate::Error> for FrontendError {
    fn from(value: crate::Error) -> Self {
        let cause = aether_core::core::domain::error_chain(&value);
        let dto = match value {
            crate::Error::AppSettingsError(err) => Self::AppSettings(err.into()),
            crate::Error::WindowError(err) => Self::Window(err.into()),
            // Raised while the app starts, before any command can carry it to the UI.
            crate::Error::LaunchError(details) => {
                Self::Application(ApplicationErrorDto::Internal {
                    cause: CauseDto::unknown(details),
                })
            }
            crate::Error::IoError(err) => Self::Application(ApplicationErrorDto::Io {
                cause: (&err).into(),
            }),
            crate::Error::IdempotencyManager(err) => Self::Application((&err).into()),
            // Logged by the conversion below.
            crate::Error::Core(core_err) => return Self::from(core_err),
        };
        dto.log(&cause);
        dto
    }
}

impl From<aether_core::Error> for FrontendError {
    fn from(value: aether_core::Error) -> Self {
        let dto = Self::from(value.raw.as_ref());
        dto.log(&value.raw.display_chain());
        dto
    }
}

impl FrontendError {
    /// For an error with no frontend category: no conversion logs it, so it is logged here.
    pub fn internal(err: impl std::fmt::Display) -> Self {
        tracing::warn!(error = %err, "Command failed");
        Self::Application(ApplicationErrorDto::Internal {
            cause: CauseDto::unknown(err),
        })
    }

    fn is_expected(&self) -> bool {
        matches!(
            self,
            Self::Instance(InstanceErrorDto::Cancelled)
                | Self::Java(JavaErrorDto::Cancelled)
                | Self::Application(ApplicationErrorDto::DuplicateRequest)
        )
    }

    fn log(&self, cause: &str) {
        if self.is_expected() {
            tracing::debug!(error = %cause, dto = ?self, "Command refused");
        } else {
            tracing::warn!(error = %cause, dto = ?self, "Command failed");
        }
    }
}

impl From<&aether_core::ErrorKind> for FrontendError {
    fn from(value: &aether_core::ErrorKind) -> Self {
        match value {
            aether_core::ErrorKind::AuthError(err) => Self::Auth(err.into()),
            aether_core::ErrorKind::EventError(err) => Self::Event(err.into()),
            aether_core::ErrorKind::FileWatcherError(err) => Self::FileWatcher(err.into()),
            aether_core::ErrorKind::GameLogError(err) => Self::GameLog(err.into()),
            aether_core::ErrorKind::InstanceError(err) => err.into(),
            aether_core::ErrorKind::JavaError(err) => err.into(),
            aether_core::ErrorKind::MinecraftError(err) => err.into(),
            aether_core::ErrorKind::PluginError(err) => Self::Plugin(err.into()),
            aether_core::ErrorKind::ProcessError(err) => Self::Process(err.into()),
            aether_core::ErrorKind::SettingsError(err) => Self::Settings(err.into()),
            aether_core::ErrorKind::RequestError(err) => Self::Request(err.into()),
            aether_core::ErrorKind::CoreError(err) => {
                Self::Application(ApplicationErrorDto::Internal {
                    cause: CauseDto::unknown(err),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_expected_when_command_was_cancelled() {
        assert!(FrontendError::Instance(InstanceErrorDto::Cancelled).is_expected());
        assert!(FrontendError::Java(JavaErrorDto::Cancelled).is_expected());
    }

    #[test]
    fn should_be_expected_when_request_is_duplicate() {
        assert!(FrontendError::Application(ApplicationErrorDto::DuplicateRequest).is_expected());
    }

    #[test]
    fn should_not_be_expected_when_error_is_internal() {
        assert!(!FrontendError::internal("x").is_expected());
    }
}
