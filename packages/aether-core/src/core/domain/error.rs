use tracing_error::InstrumentError;

use crate::{
    features::{auth, events, file_watcher, instance, java, minecraft, plugins, process, settings},
    shared::request_client,
};

/// Failure categories of the core; most variants transparently wrap a feature's own error.
#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    // Feature errors
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

    // Shared infrastructure errors
    #[error(transparent)]
    RequestError(#[from] request_client::RequestError),

    // Generic errors
    #[error("Core error: {0}")]
    CoreError(String),
}

impl ErrorKind {
    /// Wraps this kind into an [`Error`], capturing the current tracing span.
    pub fn as_error(self) -> Error {
        self.into()
    }
}

impl From<plugins::PluginProviderError> for ErrorKind {
    fn from(value: plugins::PluginProviderError) -> Self {
        Self::PluginError(value.into())
    }
}

/// Core error: the [`ErrorKind`] plus a span trace for diagnostics.
#[derive(Debug)]
pub struct Error {
    /// The underlying error kind.
    pub raw: std::sync::Arc<ErrorKind>,
    /// The same error instrumented with the current tracing span.
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

pub type Result<T> = core::result::Result<T, Error>;
