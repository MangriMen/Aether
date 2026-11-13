//! Error categories the domain keeps without knowing the library behind them.

/// A source error of a kind the domain only passes on.
pub type BoxedSource = Box<dyn std::error::Error + Send + Sync>;

/// A failure of the launcher's database. The driver's error is kept as the source; the
/// conversion from it lives with the code that talks to the database.
#[derive(Debug, thiserror::Error)]
#[error("Database error: {0}")]
pub struct DatabaseError(#[source] BoxedSource);

impl DatabaseError {
    pub fn new(source: impl Into<BoxedSource>) -> Self {
        Self(source.into())
    }
}
