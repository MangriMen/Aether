use serr::SerializeError;
use tracing_error::InstrumentError;

use crate::{
    features::{auth, events, file_watcher, instance, java, minecraft, plugins, process, settings},
    libs::request_client,
};

#[derive(thiserror::Error, Debug, SerializeError)]
pub enum ErrorKind {
    #[error(transparent)]
    #[serialize_error]
    AuthError(#[from] auth::AuthApplicationError),

    #[error(transparent)]
    #[serialize_error]
    EventError(#[from] events::EventError),

    #[error(transparent)]
    #[serialize_error]
    FileWatcherError(#[from] file_watcher::FileWatcherError),

    #[error(transparent)]
    #[serialize_error]
    InstanceError(#[from] instance::InstanceError),

    #[error(transparent)]
    #[serialize_error]
    JavaError(#[from] java::app::JavaApplicationError),

    #[error(transparent)]
    #[serialize_error]
    MinecraftError(#[from] minecraft::app::MinecraftApplicationError),

    #[error(transparent)]
    #[serialize_error]
    PluginError(#[from] plugins::PluginError),

    #[error(transparent)]
    #[serialize_error]
    ProcessError(#[from] process::ProcessError),

    #[error(transparent)]
    #[serialize_error]
    SettingsError(#[from] settings::SettingsError),

    #[error(transparent)]
    #[serialize_error]
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
