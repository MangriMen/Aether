use tracing_error::InstrumentError;

use crate::{
    features::{
        auth, events, file_watcher, game_log, instance, java, minecraft, plugins, process, settings,
    },
    shared::request_client,
};

/// Failure categories of the core; most variants transparently wrap a feature error.
#[derive(thiserror::Error, Debug, Clone)]
pub enum ErrorKind {
    #[error(transparent)]
    AuthError(#[from] auth::AuthApplicationError),

    #[error(transparent)]
    EventError(#[from] events::EventError),

    #[error(transparent)]
    FileWatcherError(#[from] file_watcher::FileWatcherError),

    #[error(transparent)]
    GameLogError(#[from] game_log::GameLogError),

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

/// Core error: the [`ErrorKind`] plus a span trace for diagnostics.
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

    /// Full cause chain for logs; starts at the kind, since `Error` repeats its text via the span trace.
    pub fn display_chain(&self) -> String {
        error_chain(self)
    }
}

const MAX_CHAIN_LEN: usize = 4096;

/// Joins an error and its `source()` chain with ": ", skipping links whose text is already in it.
pub fn error_chain(err: &(dyn std::error::Error + 'static)) -> String {
    let mut out = String::new();
    let mut current = Some(err);
    while let Some(e) = current {
        let text = e.to_string();
        // thiserror `{0}`, transparent wrappers and `{source}, path: ..` repeat the inner message.
        if !out.contains(&text) {
            if !out.is_empty() {
                out.push_str(": ");
            }
            out.push_str(&text);
        }
        current = e.source();
    }
    if let Some((idx, _)) = out.char_indices().nth(MAX_CHAIN_LEN) {
        out.truncate(idx);
    }
    out
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<minecraft::MinecraftDomainError> for ErrorKind {
    fn from(value: minecraft::MinecraftDomainError) -> Self {
        Self::MinecraftError(value.into())
    }
}

impl From<java::JavaDomainError> for ErrorKind {
    fn from(value: java::JavaDomainError) -> Self {
        Self::JavaError(value.into())
    }
}

impl From<plugins::PluginProviderError> for ErrorKind {
    fn from(value: plugins::PluginProviderError) -> Self {
        Self::PluginError(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(thiserror::Error, Debug)]
    #[error("{0}")]
    struct Wrapper(#[source] Inner);

    #[derive(thiserror::Error, Debug)]
    #[error("disk full")]
    struct Inner;

    #[test]
    fn should_keep_the_typed_source_and_the_text_when_an_error_is_cloned() {
        let original = ErrorKind::from(crate::features::instance::InstanceError::storage(
            crate::shared::io::IoError::with_path(
                std::io::Error::from(std::io::ErrorKind::StorageFull),
                "/data/instances/a",
            ),
        ));

        let clone = original.clone();

        assert_eq!(clone.display_chain(), original.display_chain());
        assert!(matches!(
            clone,
            ErrorKind::InstanceError(crate::features::instance::InstanceError::Storage(
                crate::features::instance::StorageError::SharedIo(
                    crate::shared::io::IoError::IoPathError { ref source, .. }
                )
            )) if source.kind() == std::io::ErrorKind::StorageFull
        ));
    }

    #[test]
    fn should_not_repeat_link_when_source_has_same_text() {
        assert_eq!(error_chain(&Wrapper(Inner)), "disk full");
    }

    #[derive(thiserror::Error, Debug)]
    #[error("save failed")]
    struct Outer(#[source] Inner);

    #[derive(thiserror::Error, Debug)]
    #[error("{source}, path: /x")]
    struct WithPath {
        #[source]
        source: Inner,
    }

    #[test]
    fn should_join_links_when_texts_differ() {
        assert_eq!(error_chain(&Outer(Inner)), "save failed: disk full");
    }

    #[test]
    fn should_not_repeat_source_when_text_embeds_it() {
        assert_eq!(
            error_chain(&WithPath { source: Inner }),
            "disk full, path: /x"
        );
    }

    #[test]
    fn should_truncate_to_limit_when_chain_is_too_long() {
        let kind = ErrorKind::CoreError("я".repeat(10_000));
        assert_eq!(kind.display_chain().chars().count(), MAX_CHAIN_LEN);
    }
}
