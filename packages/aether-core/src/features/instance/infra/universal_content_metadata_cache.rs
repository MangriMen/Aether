//! [`ContentMetadataCache`] over the shared key-value [`Cache`] (`universal_cache` in
//! production).
//!
//! Staleness is the app's to judge ([`CachedContentMetadata::is_stale`]), by the time an item
//! was fetched, kept inside the value. The row itself is given a much longer life
//! ([`ROW_TTL`]): the cache hides expired rows, and a stale item must stay readable — offline it
//! is the only name and icon there is. Only an item nobody refreshed for that long goes: hidden
//! at once, deleted by [`Cache::purge_expired`] at the next start.
//!
//! Versions are kept the same way, in a namespace of their own and for longer
//! ([`VERSION_ROW_TTL`]): a version does not change once published.

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    features::instance::{
        ContentMetadata, ContentVersionMetadata, ProviderId,
        app::{
            CachedContentMetadata, CachedContentVersionMetadata, CachedMetadata,
            ContentMetadataCache,
        },
    },
    shared::cache::{Cache, CacheId, CacheKey, CachedValue},
};

/// One namespace for all providers, the provider in the key: a namespace is static, provider
/// ids are not.
const NAMESPACE: &str = "content_metadata";

/// How long a row outlives its last refresh before the cache drops it.
const ROW_TTL: Duration = Duration::from_hours(90 * 24);

/// The namespace of versions, the provider in the key as in [`NAMESPACE`].
const VERSION_NAMESPACE: &str = "content_version_metadata";

/// How long a version's row outlives its last refresh: longer than the version stays fresh
/// ([`CONTENT_VERSION_METADATA_TTL`]), so a stale one stays readable.
///
/// [`CONTENT_VERSION_METADATA_TTL`]: crate::features::instance::app::CONTENT_VERSION_METADATA_TTL
const VERSION_ROW_TTL: Duration = Duration::from_hours(365 * 24);

/// [`ContentVersionMetadata`] as stored.
#[derive(Serialize, Deserialize)]
struct ContentVersionMetadataRecord {
    version_id: String,
    version_number: String,
    name: Option<String>,
}

impl From<&ContentVersionMetadata> for ContentVersionMetadataRecord {
    fn from(value: &ContentVersionMetadata) -> Self {
        Self {
            version_id: value.version_id.clone(),
            version_number: value.version_number.clone(),
            name: value.name.clone(),
        }
    }
}

impl From<ContentVersionMetadataRecord> for ContentVersionMetadata {
    fn from(value: ContentVersionMetadataRecord) -> Self {
        Self {
            version_id: value.version_id,
            version_number: value.version_number,
            name: value.name,
        }
    }
}

/// [`ContentMetadata`] as stored: the domain stays free of serde.
#[derive(Serialize, Deserialize)]
struct ContentMetadataRecord {
    content_id: String,
    slug: String,
    name: String,
    description: Option<String>,
    author: Option<String>,
    icon_url: Option<String>,
}

impl From<&ContentMetadata> for ContentMetadataRecord {
    fn from(value: &ContentMetadata) -> Self {
        Self {
            content_id: value.content_id.clone(),
            slug: value.slug.clone(),
            name: value.name.clone(),
            description: value.description.clone(),
            author: value.author.clone(),
            icon_url: value.icon_url.clone(),
        }
    }
}

impl From<ContentMetadataRecord> for ContentMetadata {
    fn from(value: ContentMetadataRecord) -> Self {
        Self {
            content_id: value.content_id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            author: value.author,
            icon_url: value.icon_url,
        }
    }
}

/// `<provider>:<id>` in `namespace`. A provider id has no `:` ([`is_valid_id`] parts joined by
/// `#`), so no two pairs share a key.
///
/// [`is_valid_id`]: crate::features::instance::is_valid_id
fn key<R>(namespace: &'static str, provider_id: &ProviderId, id: &str) -> CacheKey<CachedValue<R>> {
    CacheKey::new(namespace, CacheId::Named(format!("{provider_id}:{id}")))
}

pub struct UniversalContentMetadataCache<C> {
    cache: C,
}

impl<C: Cache> UniversalContentMetadataCache<C> {
    pub fn new(cache: C) -> Self {
        Self { cache }
    }

    /// The rows of `ids` found in `namespace` for `provider_id`, as items of type `T`, keyed by
    /// id. An id with no row is absent.
    async fn get_records<R, T>(
        &self,
        namespace: &'static str,
        provider_id: &ProviderId,
        ids: &[String],
    ) -> HashMap<String, CachedMetadata<T>>
    where
        R: DeserializeOwned + Send + Sync + Into<T>,
    {
        let keys: Vec<_> = ids
            .iter()
            .map(|id| key::<R>(namespace, provider_id, id))
            .collect();
        let found = self.cache.get_many(&keys).await;

        ids.iter()
            .zip(found)
            .filter_map(|(id, cached)| {
                let cached = cached?;
                Some((
                    id.clone(),
                    CachedMetadata {
                        metadata: cached.value.into(),
                        fetched_at: cached.updated_at,
                    },
                ))
            })
            .collect()
    }

    /// Put `records` of `provider_id` in `namespace`, each under the id it is paired with, as
    /// fetched at `fetched_at`, the rows to live `row_ttl`; replacing what was there.
    async fn put_records<R>(
        &self,
        namespace: &'static str,
        row_ttl: Duration,
        provider_id: &ProviderId,
        records: Vec<(&str, R)>,
        fetched_at: SystemTime,
    ) where
        R: Serialize + Send + Sync,
    {
        let entries: Vec<_> = records
            .into_iter()
            .map(|(id, record)| {
                let value = CachedValue {
                    value: record,
                    updated_at: fetched_at,
                };
                (key(namespace, provider_id, id), value)
            })
            .collect();
        self.cache.set_many(&entries, row_ttl).await;
    }

    async fn put_at(
        &self,
        provider_id: &ProviderId,
        items: &[(String, ContentMetadata)],
        fetched_at: SystemTime,
    ) {
        let records = items
            .iter()
            .map(|(content_id, item)| (content_id.as_str(), ContentMetadataRecord::from(item)))
            .collect();
        self.put_records(NAMESPACE, ROW_TTL, provider_id, records, fetched_at)
            .await;
    }

    async fn put_versions_at(
        &self,
        provider_id: &ProviderId,
        items: &[ContentVersionMetadata],
        fetched_at: SystemTime,
    ) {
        let records = items
            .iter()
            .map(|item| {
                let record = ContentVersionMetadataRecord::from(item);
                (item.version_id.as_str(), record)
            })
            .collect();
        self.put_records(
            VERSION_NAMESPACE,
            VERSION_ROW_TTL,
            provider_id,
            records,
            fetched_at,
        )
        .await;
    }
}

#[async_trait]
impl<C: Cache> ContentMetadataCache for UniversalContentMetadataCache<C> {
    async fn get_many(
        &self,
        provider_id: &ProviderId,
        content_ids: &[String],
    ) -> HashMap<String, CachedContentMetadata> {
        self.get_records::<ContentMetadataRecord, _>(NAMESPACE, provider_id, content_ids)
            .await
    }

    async fn put_keyed(&self, provider_id: &ProviderId, items: &[(String, ContentMetadata)]) {
        self.put_at(provider_id, items, SystemTime::now()).await;
    }

    async fn get_versions(
        &self,
        provider_id: &ProviderId,
        version_ids: &[String],
    ) -> HashMap<String, CachedContentVersionMetadata> {
        self.get_records::<ContentVersionMetadataRecord, _>(
            VERSION_NAMESPACE,
            provider_id,
            version_ids,
        )
        .await
    }

    async fn put_versions(&self, provider_id: &ProviderId, items: &[ContentVersionMetadata]) {
        self.put_versions_at(provider_id, items, SystemTime::now())
            .await;
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, pool::PoolOptions};

    use crate::{
        core::migrations::run_migrations,
        features::instance::app::{CONTENT_METADATA_TTL, CONTENT_VERSION_METADATA_TTL},
        shared::cache::infra::SqliteCache,
    };

    use super::*;

    async fn create_cache() -> UniversalContentMetadataCache<SqliteCache> {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);
        let pool: SqlitePool = PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to create in-memory SQLite pool");
        run_migrations(&pool).await;

        UniversalContentMetadataCache::new(SqliteCache::new(pool))
    }

    fn provider(capability_id: &str) -> ProviderId {
        ProviderId {
            plugin_id: "aether".into(),
            capability_id: capability_id.into(),
        }
    }

    fn metadata(content_id: &str, name: &str) -> ContentMetadata {
        ContentMetadata {
            content_id: content_id.into(),
            slug: name.to_lowercase(),
            name: name.into(),
            description: Some("desc".into()),
            author: None,
            icon_url: Some(format!(
                "https://cdn.modrinth.com/data/{content_id}/abc.png"
            )),
        }
    }

    fn keyed(item: ContentMetadata) -> (String, ContentMetadata) {
        (item.content_id.clone(), item)
    }

    fn ids(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|&id| id.to_owned()).collect()
    }

    #[tokio::test]
    async fn should_return_only_the_cached_items_when_some_ids_are_missing() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        cache
            .put_many(
                &modrinth,
                &[
                    metadata("AANobbMI", "Sodium"),
                    metadata("P7dR8mSH", "Fabric API"),
                ],
            )
            .await;

        let found = cache
            .get_many(&modrinth, &ids(&["AANobbMI", "unknown", "P7dR8mSH"]))
            .await;

        assert_eq!(found.len(), 2);
        assert_eq!(found["AANobbMI"].metadata, metadata("AANobbMI", "Sodium"));
        assert_eq!(found["P7dR8mSH"].metadata.name, "Fabric API");
        assert!(!found["AANobbMI"].is_stale(SystemTime::now()));
    }

    #[tokio::test]
    async fn should_keep_providers_apart_when_they_share_a_content_id() {
        let cache = create_cache().await;
        cache
            .put_many(&provider("modrinth"), &[metadata("same", "Modrinth one")])
            .await;
        cache
            .put_many(&provider("other"), &[metadata("same", "Other one")])
            .await;

        let modrinth = cache.get_many(&provider("modrinth"), &ids(&["same"])).await;
        let other = cache.get_many(&provider("other"), &ids(&["same"])).await;
        let none = cache.get_many(&provider("third"), &ids(&["same"])).await;

        assert_eq!(modrinth["same"].metadata.name, "Modrinth one");
        assert_eq!(other["same"].metadata.name, "Other one");
        assert!(none.is_empty());
    }

    #[tokio::test]
    async fn should_still_return_an_item_when_it_is_stale() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        let fetched_at = SystemTime::now() - CONTENT_METADATA_TTL - Duration::from_hours(24);
        cache
            .put_at(
                &modrinth,
                &[keyed(metadata("AANobbMI", "Sodium"))],
                fetched_at,
            )
            .await;

        let found = cache.get_many(&modrinth, &ids(&["AANobbMI"])).await;

        let item = &found["AANobbMI"];
        assert_eq!(item.metadata.name, "Sodium");
        assert_eq!(item.fetched_at, fetched_at);
        assert!(item.is_stale(SystemTime::now()));
    }

    #[tokio::test]
    async fn should_replace_an_item_and_renew_it_when_put_again() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        let long_ago = SystemTime::now() - CONTENT_METADATA_TTL - Duration::from_hours(24);
        cache
            .put_at(
                &modrinth,
                &[keyed(metadata("AANobbMI", "Old name"))],
                long_ago,
            )
            .await;

        cache
            .put_many(&modrinth, &[metadata("AANobbMI", "Sodium")])
            .await;
        let found = cache.get_many(&modrinth, &ids(&["AANobbMI"])).await;

        assert_eq!(found["AANobbMI"].metadata.name, "Sodium");
        assert!(!found["AANobbMI"].is_stale(SystemTime::now()));
    }

    #[tokio::test]
    async fn should_find_an_item_by_the_id_it_was_put_under_when_that_is_not_its_own() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        let sodium = metadata("AANobbMI", "Sodium");

        cache
            .put_keyed(&modrinth, &[("sodium".to_owned(), sodium.clone())])
            .await;
        let found = cache
            .get_many(&modrinth, &ids(&["sodium", "AANobbMI"]))
            .await;

        assert_eq!(found.len(), 1);
        assert_eq!(found["sodium"].metadata, sodium);
    }

    fn version(version_id: &str, version_number: &str) -> ContentVersionMetadata {
        ContentVersionMetadata {
            version_id: version_id.into(),
            version_number: version_number.into(),
            name: Some(format!("Sodium {version_number}")),
        }
    }

    #[tokio::test]
    async fn should_return_only_the_cached_versions_when_some_ids_are_missing() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        cache
            .put_versions(&modrinth, &[version("IZskON6d", "0.5.8")])
            .await;

        let found = cache
            .get_versions(&modrinth, &ids(&["IZskON6d", "unknown"]))
            .await;
        let other = cache
            .get_versions(&provider("other"), &ids(&["IZskON6d"]))
            .await;

        assert_eq!(found.len(), 1);
        assert_eq!(found["IZskON6d"].metadata, version("IZskON6d", "0.5.8"));
        assert!(!found["IZskON6d"].is_stale(SystemTime::now()));
        assert!(other.is_empty());
    }

    #[tokio::test]
    async fn should_keep_versions_apart_from_projects_when_they_share_an_id() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        cache
            .put_many(&modrinth, &[metadata("same", "Sodium")])
            .await;
        cache
            .put_versions(&modrinth, &[version("same", "0.5.8")])
            .await;

        let projects = cache.get_many(&modrinth, &ids(&["same"])).await;
        let versions = cache.get_versions(&modrinth, &ids(&["same"])).await;

        assert_eq!(projects["same"].metadata.name, "Sodium");
        assert_eq!(versions["same"].metadata.version_number, "0.5.8");
    }

    #[tokio::test]
    async fn should_still_return_a_version_when_it_is_stale() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        let fetched_at =
            SystemTime::now() - CONTENT_VERSION_METADATA_TTL - Duration::from_hours(24);
        cache
            .put_versions_at(&modrinth, &[version("IZskON6d", "0.5.8")], fetched_at)
            .await;

        let found = cache.get_versions(&modrinth, &ids(&["IZskON6d"])).await;

        let item = &found["IZskON6d"];
        assert_eq!(item.metadata.version_number, "0.5.8");
        assert_eq!(item.fetched_at, fetched_at);
        assert!(item.is_stale(SystemTime::now()));
    }
}
