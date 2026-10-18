/// What the user sees of the caches may have changed: they were emptied, cleaned in the
/// background, or filled with what a content list fetched.
#[derive(Debug, Clone)]
pub enum CacheEvent {
    Changed,
}
