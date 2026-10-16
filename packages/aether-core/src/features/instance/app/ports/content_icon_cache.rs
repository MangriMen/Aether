use std::collections::HashSet;

use async_trait::async_trait;

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
        /// Changes whenever another icon may have been put under `asset_id`; `None` when that
        /// never happens. What the UI shows the icon by carries it, or it would keep showing
        /// the copy it cached of the old one.
        revision: Option<u64>,
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
    /// What is cached for `url`.
    async fn lookup(&self, url: &str) -> CachedContentIcon;

    /// Fetch the icon of `url`, or ask whether the cached one changed. `Ok(true)` when a new
    /// file is in place, `Ok(false)` when the cached one is still current.
    async fn fetch(&self, url: &str) -> Result<bool, ContentIconError>;

    /// Delete the icons of URLs not in `keep`, and what an interrupted fetch left behind. An
    /// icon fetched just now stays whatever `keep` says: its owner may not be in it yet. How
    /// many icons went.
    async fn retain(&self, keep: &HashSet<String>) -> usize;
}
