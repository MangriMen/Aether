//! Error categories the domain keeps without knowing the library behind them.

use std::sync::Arc;

/// A source error of a kind the domain only passes on, as it is handed over.
pub type BoxedSource = Box<dyn std::error::Error + Send + Sync>;

/// A source error kept by an error that is cloned: the clones share it.
pub type SharedSource = Arc<dyn std::error::Error + Send + Sync>;

/// `source` ready to be kept by an error that is cloned.
pub fn shared_source(source: impl Into<BoxedSource>) -> SharedSource {
    Arc::from(source.into())
}

/// A failure of the launcher's database. The driver's error is kept as the source; the
/// conversion from it lives with the code that talks to the database.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Database error: {0}")]
pub struct DatabaseError(#[source] SharedSource);

impl DatabaseError {
    pub fn new(source: impl Into<BoxedSource>) -> Self {
        Self(shared_source(source))
    }
}

/// An `anyhow::Error` kept as a source that clones share. Its chain starts at the error it
/// wraps, so a typed cause in it is found by walking `source()`.
#[derive(Debug, Clone)]
pub struct AnyhowSource(Arc<anyhow::Error>);

impl AnyhowSource {
    pub fn new(err: anyhow::Error) -> Self {
        Self(Arc::new(err))
    }

    pub fn inner(&self) -> &anyhow::Error {
        &self.0
    }
}

impl From<anyhow::Error> for AnyhowSource {
    fn from(err: anyhow::Error) -> Self {
        Self::new(err)
    }
}

impl std::fmt::Display for AnyhowSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for AnyhowSource {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref().as_ref())
    }
}
