use aether_core::shared::error::SharedSource;

/// Why an update could not be checked for or installed. The updater's error is kept as the
/// source, shared by the clones.
#[derive(Debug, Clone, thiserror::Error)]
pub enum UpdateError {
    /// An install was asked for with no update found by a check before it.
    #[error("No update to install: check for updates first")]
    NoPendingUpdate,

    #[error("Could not download the update: {0}")]
    Network(#[source] SharedSource),

    /// The update is not signed as the launcher expects, so it was not installed.
    #[error("The update signature is invalid: {0}")]
    Signature(#[source] SharedSource),

    #[error("Could not install the update: {0}")]
    Install(#[source] SharedSource),

    #[error("Could not check for updates: {0}")]
    Other(#[source] SharedSource),
}
