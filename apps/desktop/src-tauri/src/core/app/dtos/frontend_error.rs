use serde::Serialize;
use specta::Type;

use crate::core::{
    AppSettingsErrorDto, AuthErrorDto, EventErrorDto, FileWatcherErrorDto, InstanceErrorDto,
    JavaErrorDto, MinecraftErrorDto, PluginErrorDto, ProcessErrorDto, RequestErrorDto,
    SettingsErrorDto,
};

pub type FrontendResult<T, E = FrontendError> = Result<T, E>;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum FrontendError {
    AppSettings(AppSettingsErrorDto),

    Auth(AuthErrorDto),

    Event(EventErrorDto),

    FileWatcher(FileWatcherErrorDto),

    Instance(InstanceErrorDto),

    Java(JavaErrorDto),

    Minecraft(MinecraftErrorDto),

    Plugin(PluginErrorDto),

    Process(ProcessErrorDto),

    Request(RequestErrorDto),

    Settings(SettingsErrorDto),

    /// General errors that are expected and can be shown to the user.
    Generic(String),

    /// Critical or unexpected errors that indicate a logic failure or
    /// improper error mapping. These should ideally never reach the frontend
    /// but serve as a safety net to prevent silent crashes.
    Internal(String),
}

impl From<crate::Error> for FrontendError {
    fn from(value: crate::Error) -> Self {
        match value {
            crate::Error::AppSettingsError(err) => Self::AppSettings(err.into()),
            crate::Error::LaunchError(err) => Self::Internal(err),
            crate::Error::IoError(err) => Self::Generic(err.to_string()),
            crate::Error::Core(core_err) => Self::from(core_err.raw.as_ref()),
        }
    }
}

impl From<&aether_core::ErrorKind> for FrontendError {
    fn from(value: &aether_core::ErrorKind) -> Self {
        match value {
            aether_core::ErrorKind::AuthError(err) => Self::Auth(err.into()),
            aether_core::ErrorKind::EventError(err) => Self::Event(err.into()),
            aether_core::ErrorKind::FileWatcherError(err) => Self::FileWatcher(err.into()),
            aether_core::ErrorKind::InstanceError(err) => Self::Instance(err.into()),
            aether_core::ErrorKind::JavaError(err) => Self::Java(err.into()),
            aether_core::ErrorKind::MinecraftError(err) => Self::Minecraft(err.into()),
            aether_core::ErrorKind::PluginError(err) => Self::Plugin(err.into()),
            aether_core::ErrorKind::ProcessError(err) => Self::Process(err.into()),
            aether_core::ErrorKind::SettingsError(err) => Self::Settings(err.into()),
            aether_core::ErrorKind::RequestError(err) => Self::Request(err.into()),
            aether_core::ErrorKind::CoreError(err) => Self::Generic(err.to_owned()),
        }
    }
}
