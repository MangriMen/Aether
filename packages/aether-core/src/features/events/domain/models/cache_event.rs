/// What the UI shows of the caches.
#[derive(Debug, Clone)]
pub enum CacheEvent {
    /// What the user sees of the caches may have changed: they were emptied, cleaned in the
    /// background, or filled with what a content list fetched.
    Changed,
    /// Whether the app is busy with something that uses the caches (installs, downloads,
    /// emptying) changed, so clearing is refused or allowed again.
    BusyChanged { busy: bool },
}
