use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use async_trait::async_trait;

use crate::features::instance::domain::{ContentMetadata, ProviderId};

/// How long [`ContentMetadata`] is taken as current. Past it, the cached item is still shown —
/// offline it is all there is — but asked of the provider again.
pub const CONTENT_METADATA_TTL: Duration = Duration::from_hours(7 * 24);

/// [`ContentMetadata`] as cached, with when it was fetched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedContentMetadata {
    pub metadata: ContentMetadata,
    pub fetched_at: SystemTime,
}

impl CachedContentMetadata {
    /// Whether the item is older than [`CONTENT_METADATA_TTL`] as of `now`, and should be asked
    /// of the provider again. A stale item is still good to show until then.
    pub fn is_stale(&self, now: SystemTime) -> bool {
        now.duration_since(self.fetched_at)
            .is_ok_and(|age| age > CONTENT_METADATA_TTL)
    }
}

/// The [`ContentMetadata`] the providers gave, kept by provider and content id so an instance's
/// content list shows names and icons without asking the provider each time, and offline.
///
/// Stale items are kept and returned (see [`CachedContentMetadata::is_stale`]): a failed
/// refresh leaves them as they were. Best effort, as any cache: a failure to read is a miss,
/// a failure to write is dropped.
#[async_trait]
pub trait ContentMetadataCache: Send + Sync {
    /// The cached items of `content_ids` found for `provider_id`, fresh and stale alike, keyed
    /// by content id. An id with nothing cached is absent.
    async fn get_many(
        &self,
        provider_id: &ProviderId,
        content_ids: &[String],
    ) -> HashMap<String, CachedContentMetadata>;

    /// Cache `items` of `provider_id` as fetched just now, each under its
    /// [`ContentMetadata::content_id`], replacing what was there.
    async fn put_many(&self, provider_id: &ProviderId, items: &[ContentMetadata]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached(fetched_at: SystemTime) -> CachedContentMetadata {
        CachedContentMetadata {
            metadata: ContentMetadata {
                content_id: "AANobbMI".into(),
                slug: "sodium".into(),
                name: "Sodium".into(),
                description: None,
                author: None,
                icon_url: None,
            },
            fetched_at,
        }
    }

    #[test]
    fn should_be_stale_only_when_older_than_the_ttl() {
        let now = SystemTime::now();

        assert!(!cached(now).is_stale(now));
        assert!(!cached(now - CONTENT_METADATA_TTL).is_stale(now));
        assert!(cached(now - CONTENT_METADATA_TTL - Duration::from_secs(1)).is_stale(now));
        // Fetched "in the future" (the clock went back): not stale.
        assert!(!cached(now + Duration::from_hours(1)).is_stale(now));
    }
}
