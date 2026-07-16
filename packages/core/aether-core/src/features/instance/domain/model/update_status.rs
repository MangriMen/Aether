/// Result of a `PackManager::check_updates` call.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UpdateStatus {
    /// Instance is up to date.
    NoUpdates,
    /// A new version is available.
    UpdatesAvailable {
        current_version: String,
        latest_version: String,
    },
    /// Update checking is not supported by this provider.
    Unsupported,
}
