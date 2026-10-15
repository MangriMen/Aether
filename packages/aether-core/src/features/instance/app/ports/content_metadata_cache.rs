use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use async_trait::async_trait;

use crate::features::instance::domain::{ContentMetadata, ContentVersionMetadata, ProviderId};

/// How long [`ContentMetadata`] is taken as current. Past it, the cached item is still shown —
/// offline it is all there is — but asked of the provider again.
pub const CONTENT_METADATA_TTL: Duration = Duration::from_hours(7 * 24);

/// How long [`ContentVersionMetadata`] is taken as current. A version does not change once
/// published, so this is long; past it, the item is shown as [`CONTENT_METADATA_TTL`] says.
pub const CONTENT_VERSION_METADATA_TTL: Duration = Duration::from_hours(90 * 24);

/// An item as cached, with when it was fetched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedMetadata<T> {
    pub metadata: T,
    pub fetched_at: SystemTime,
}

impl<T> CachedMetadata<T> {
    /// Whether the item is older than `ttl` as of `now`. Fetched "in the future" (the clock
    /// went back), it is not.
    fn is_older_than(&self, ttl: Duration, now: SystemTime) -> bool {
        now.duration_since(self.fetched_at)
            .is_ok_and(|age| age > ttl)
    }
}

/// [`ContentMetadata`] as cached, with when it was fetched.
pub type CachedContentMetadata = CachedMetadata<ContentMetadata>;

impl CachedContentMetadata {
    /// Whether the item is older than [`CONTENT_METADATA_TTL`] as of `now`, and should be asked
    /// of the provider again. A stale item is still good to show until then.
    pub fn is_stale(&self, now: SystemTime) -> bool {
        self.is_older_than(CONTENT_METADATA_TTL, now)
    }
}

/// [`ContentVersionMetadata`] as cached, with when it was fetched.
pub type CachedContentVersionMetadata = CachedMetadata<ContentVersionMetadata>;

impl CachedContentVersionMetadata {
    /// Whether the item is older than [`CONTENT_VERSION_METADATA_TTL`] as of `now`, and should
    /// be asked of the provider again. A stale item is still good to show until then.
    pub fn is_stale(&self, now: SystemTime) -> bool {
        self.is_older_than(CONTENT_VERSION_METADATA_TTL, now)
    }
}

/// The [`ContentMetadata`] the providers gave, kept by provider and content id so an instance's
/// content list shows names and icons without asking the provider each time, and offline; and
/// the [`ContentVersionMetadata`], kept by provider and version id, to show versions by.
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

    /// Cache `items` of `provider_id` as fetched just now, each under the content id it is
    /// paired with, replacing what was there.
    ///
    /// The id need not be the item's own [`ContentMetadata::content_id`]: an item the provider
    /// found by another id (a slug) is put under that one too, so asking by it again hits.
    async fn put_keyed(&self, provider_id: &ProviderId, items: &[(String, ContentMetadata)]);

    /// Cache `items` of `provider_id` as fetched just now, each under its own
    /// [`ContentMetadata::content_id`], replacing what was there.
    async fn put_many(&self, provider_id: &ProviderId, items: &[ContentMetadata]) {
        let keyed: Vec<_> = items
            .iter()
            .map(|item| (item.content_id.clone(), item.clone()))
            .collect();
        self.put_keyed(provider_id, &keyed).await;
    }

    /// The cached versions of `version_ids` found for `provider_id`, fresh and stale alike,
    /// keyed by version id. An id with nothing cached is absent.
    async fn get_versions(
        &self,
        provider_id: &ProviderId,
        version_ids: &[String],
    ) -> HashMap<String, CachedContentVersionMetadata>;

    /// Cache `items` of `provider_id` as fetched just now, each under its own
    /// [`ContentVersionMetadata::version_id`], replacing what was there.
    async fn put_versions(&self, provider_id: &ProviderId, items: &[ContentVersionMetadata]);
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

    fn cached_version(fetched_at: SystemTime) -> CachedContentVersionMetadata {
        CachedContentVersionMetadata {
            metadata: ContentVersionMetadata {
                version_id: "IZskON6d".into(),
                version_number: "mc1.20.6-0.5.8".into(),
                name: None,
            },
            fetched_at,
        }
    }

    #[test]
    fn should_keep_a_version_fresh_when_a_project_of_the_same_age_is_stale() {
        let now = SystemTime::now();
        let ttl = CONTENT_VERSION_METADATA_TTL;

        assert!(ttl > CONTENT_METADATA_TTL);
        assert!(!cached_version(now - CONTENT_METADATA_TTL * 2).is_stale(now));
        assert!(!cached_version(now - ttl).is_stale(now));
        assert!(cached_version(now - ttl - Duration::from_secs(1)).is_stale(now));
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
