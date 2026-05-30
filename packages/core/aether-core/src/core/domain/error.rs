use tracing_error::InstrumentError;

use crate::{
    features::{auth, events, file_watcher, instance, java, minecraft, plugins, process, settings},
    shared::request_client,
};

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error(transparent)]
    AuthError(#[from] auth::AuthApplicationError),

    #[error(transparent)]
    EventError(#[from] events::EventError),

    #[error(transparent)]
    FileWatcherError(#[from] file_watcher::FileWatcherError),

    #[error(transparent)]
    InstanceError(#[from] instance::InstanceError),

    #[error(transparent)]
    JavaError(#[from] java::JavaApplicationError),

    #[error(transparent)]
    MinecraftError(#[from] minecraft::MinecraftApplicationError),

    #[error(transparent)]
    PluginError(#[from] plugins::PluginError),

    #[error(transparent)]
    ProcessError(#[from] process::ProcessError),

    #[error(transparent)]
    SettingsError(#[from] settings::SettingsError),

    #[error(transparent)]
    RequestError(#[from] request_client::RequestError),

    #[error("Core error: {0}")]
    CoreError(String),
}

#[derive(Debug)]
pub struct Error {
    pub raw: std::sync::Arc<ErrorKind>,
    pub source: tracing_error::TracedError<std::sync::Arc<ErrorKind>>,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.source)
    }
}

impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(source: E) -> Self {
        let error = Into::<ErrorKind>::into(source);
        let boxed_error = std::sync::Arc::new(error);

        Self {
            raw: boxed_error.clone(),
            source: boxed_error.in_current_span(),
        }
    }
}

impl ErrorKind {
    pub fn as_error(self) -> Error {
        self.into()
    }
}

pub type Result<T> = core::result::Result<T, Error>;
