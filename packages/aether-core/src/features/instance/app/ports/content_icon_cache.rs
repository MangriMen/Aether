use std::time::Duration;

use async_trait::async_trait;

/// How long a cached icon nobody listed is kept. Only cleanup goes by it: a kept icon is
/// shown however old, as offline it is the only one there is.
pub const CONTENT_ICON_MAX_UNUSED: Duration = Duration::from_hours(30 * 24);

/// What the icon cache holds for an icon URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CachedContentIcon {
    /// No file: the icon is to be fetched.
    Missing,
    /// A file, to be shown as it is.
    Present {
        /// The file, as an id of the asset store: its path relative to the store,
        /// `/`-separated.
        asset_id: String,
        /// Whether the server should be asked if the icon changed: a URL that does not name
        /// its content by hash may get another icon under the same name.
        revalidate: bool,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum ContentIconError {
    #[error("Not an icon URL: {0}")]
    InvalidUrl(String),

    #[error("Failed to fetch the icon: {0}")]
    Request(String),

    #[error("The response is not an image")]
    NotAnImage,

    #[error("Failed to store the icon: {0}")]
    Storage(String),
}

/// Local copies of the icons of content projects (see
/// [`ContentMetadata::icon_url`](crate::features::instance::ContentMetadata::icon_url)), kept
/// in the asset store so the UI shows them without the network, by the URL they came from.
///
/// Best effort, as any cache: a failure to read is a miss. A failed fetch keeps the file that
/// was there.
#[async_trait]
pub trait ContentIconCache: Send + Sync {
    /// What is cached for `url`. A present icon is marked as used just now, for
    /// [`Self::remove_unused`].
    async fn lookup(&self, url: &str) -> CachedContentIcon;

    /// Fetch the icon of `url`, or ask whether the cached one changed. `Ok(true)` when a new
    /// file is in place, `Ok(false)` when the cached one is still current.
    async fn fetch(&self, url: &str) -> Result<bool, ContentIconError>;

    /// Delete the icons not used for longer than `max_unused`, and what an interrupted fetch
    /// left behind. How many icons went.
    async fn remove_unused(&self, max_unused: Duration) -> usize;
}
