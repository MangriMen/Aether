//! [`ContentMetadataCache`] over the shared key-value [`Cache`] (`universal_cache` in
//! production).
//!
//! Staleness is the app's to judge ([`CachedContentMetadata::is_stale`]), by the time an item
//! was fetched, kept inside the value. The row itself does not expire ([`ROW_TTL`]): the cache
//! hides expired rows, and a stale item must stay readable however long the app was offline —
//! then it is the only name and icon there is. A row is a few hundred bytes; what is no longer
//! installed is left for the user to clear.
//!
//! Versions are kept the same way, in a namespace of their own; and so are authors.

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    features::instance::{
        ContentAuthor, ContentMetadata, ContentVersionMetadata, ProviderId,
        app::{
            CachedContentAuthor, CachedContentMetadata, CachedContentVersionMetadata,
            CachedMetadata, ContentMetadataCache,
        },
    },
    shared::cache::{Cache, CacheId, CacheKey, CachedValue},
};

/// One namespace for all providers, the provider in the key: a namespace is static, provider
/// ids are not.
const NAMESPACE: &str = "content_metadata";

/// The life of a row, of projects and of versions alike: in effect for ever. Not
/// [`Duration::MAX`]: the cache turns a life it cannot add to the clock into none.
const ROW_TTL: Duration = Duration::from_hours(100 * 365 * 24);

/// The namespace of versions, the provider in the key as in [`NAMESPACE`].
const VERSION_NAMESPACE: &str = "content_version_metadata";

/// The namespace of authors, the provider in the key as in [`NAMESPACE`].
const AUTHOR_NAMESPACE: &str = "content_author_metadata";

/// Every namespace this cache stores in, for the app to let the user clear them.
pub const NAMESPACES: [&str; 3] = [NAMESPACE, VERSION_NAMESPACE, AUTHOR_NAMESPACE];

/// [`ContentAuthor`] as stored.
#[derive(Serialize, Deserialize)]
struct ContentAuthorRecord {
    author_ref: String,
    name: String,
}

impl From<&ContentAuthor> for ContentAuthorRecord {
    fn from(value: &ContentAuthor) -> Self {
        Self {
            author_ref: value.author_ref.clone(),
            name: value.name.clone(),
        }
    }
}

impl From<ContentAuthorRecord> for ContentAuthor {
    fn from(value: ContentAuthorRecord) -> Self {
        Self {
            author_ref: value.author_ref,
            name: value.name,
        }
    }
}

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
    /// Absent from the rows cached before authors were looked up apart.
    #[serde(default)]
    author_ref: Option<String>,
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
            author_ref: value.author_ref.clone(),
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
            author_ref: value.author_ref,
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
    /// fetched at `fetched_at`; replacing what was there. Whether they were written.
    async fn put_records<R>(
        &self,
        namespace: &'static str,
        provider_id: &ProviderId,
        records: Vec<(&str, R)>,
        fetched_at: SystemTime,
    ) -> bool
    where
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
        self.cache.set_many(&entries, ROW_TTL).await
    }

    async fn put_versions_at(
        &self,
        provider_id: &ProviderId,
        items: &[ContentVersionMetadata],
        fetched_at: SystemTime,
    ) -> bool {
        let records = items
            .iter()
            .map(|item| {
                let record = ContentVersionMetadataRecord::from(item);
                (item.version_id.as_str(), record)
            })
            .collect();
        self.put_records(VERSION_NAMESPACE, provider_id, records, fetched_at)
            .await
    }

    async fn put_authors_at(
        &self,
        provider_id: &ProviderId,
        items: &[ContentAuthor],
        fetched_at: SystemTime,
    ) -> bool {
        let records = items
            .iter()
            .map(|item| (item.author_ref.as_str(), ContentAuthorRecord::from(item)))
            .collect();
        self.put_records(AUTHOR_NAMESPACE, provider_id, records, fetched_at)
            .await
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

    async fn put_keyed_at(
        &self,
        provider_id: &ProviderId,
        items: &[(String, ContentMetadata)],
        fetched_at: SystemTime,
    ) -> bool {
        let records = items
            .iter()
            .map(|(content_id, item)| (content_id.as_str(), ContentMetadataRecord::from(item)))
            .collect();
        self.put_records(NAMESPACE, provider_id, records, fetched_at)
            .await
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

    async fn put_versions(
        &self,
        provider_id: &ProviderId,
        items: &[ContentVersionMetadata],
    ) -> bool {
        self.put_versions_at(provider_id, items, SystemTime::now())
            .await
    }

    async fn get_authors(
        &self,
        provider_id: &ProviderId,
        author_refs: &[String],
    ) -> HashMap<String, CachedContentAuthor> {
        self.get_records::<ContentAuthorRecord, _>(AUTHOR_NAMESPACE, provider_id, author_refs)
            .await
    }

    async fn put_authors(&self, provider_id: &ProviderId, items: &[ContentAuthor]) -> bool {
        self.put_authors_at(provider_id, items, SystemTime::now())
            .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, pool::PoolOptions};

    use crate::{
        core::migrations::run_migrations,
        features::instance::app::{
            CONTENT_AUTHOR_TTL, CONTENT_METADATA_TTL, CONTENT_VERSION_METADATA_TTL,
        },
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
        run_migrations(&pool).await.unwrap();

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
            author_ref: Some(format!("team:{content_id}")),
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
            .put_keyed_at(
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
            .put_keyed_at(
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

    /// A row cached before authors were looked up apart has no `author_ref`.
    #[tokio::test]
    async fn should_still_load_a_project_when_it_was_cached_without_an_author_ref() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        let old_row = serde_json::json!({
            "content_id": "AANobbMI", "slug": "sodium", "name": "Sodium",
            "description": null, "author": "jellysquid3", "icon_url": null
        });
        cache
            .put_records(
                NAMESPACE,
                &modrinth,
                vec![("AANobbMI", old_row)],
                SystemTime::now(),
            )
            .await;

        let found = cache.get_many(&modrinth, &ids(&["AANobbMI"])).await;

        let item = &found["AANobbMI"].metadata;
        assert_eq!(item.name, "Sodium");
        assert_eq!(item.author.as_deref(), Some("jellysquid3"));
        assert_eq!(item.author_ref, None);
    }

    fn author(author_ref: &str, name: &str) -> ContentAuthor {
        ContentAuthor {
            author_ref: author_ref.into(),
            name: name.into(),
        }
    }

    #[tokio::test]
    async fn should_return_only_the_cached_authors_when_some_refs_are_missing() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        cache
            .put_authors(
                &modrinth,
                &[
                    author("team:abc", "jellysquid3"),
                    author("organization:xyz", "CaffeineMC"),
                ],
            )
            .await;

        let found = cache
            .get_authors(
                &modrinth,
                &ids(&["team:abc", "team:gone", "organization:xyz"]),
            )
            .await;
        let other = cache
            .get_authors(&provider("other"), &ids(&["team:abc"]))
            .await;

        assert_eq!(found.len(), 2);
        assert_eq!(
            found["team:abc"].metadata,
            author("team:abc", "jellysquid3")
        );
        assert_eq!(found["organization:xyz"].metadata.name, "CaffeineMC");
        assert!(!found["team:abc"].is_stale(SystemTime::now()));
        assert!(other.is_empty());
    }

    #[tokio::test]
    async fn should_keep_authors_apart_from_projects_when_they_share_an_id() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        cache
            .put_many(&modrinth, &[metadata("same", "Sodium")])
            .await;
        cache
            .put_authors(&modrinth, &[author("same", "jellysquid3")])
            .await;

        let projects = cache.get_many(&modrinth, &ids(&["same"])).await;
        let authors = cache.get_authors(&modrinth, &ids(&["same"])).await;

        assert_eq!(projects["same"].metadata.name, "Sodium");
        assert_eq!(authors["same"].metadata.name, "jellysquid3");
    }

    #[tokio::test]
    async fn should_still_return_an_author_when_it_is_stale() {
        let cache = create_cache().await;
        let modrinth = provider("modrinth");
        let fetched_at = SystemTime::now() - CONTENT_AUTHOR_TTL - Duration::from_hours(24);
        cache
            .put_authors_at(&modrinth, &[author("team:abc", "jellysquid3")], fetched_at)
            .await;

        let found = cache.get_authors(&modrinth, &ids(&["team:abc"])).await;

        let item = &found["team:abc"];
        assert_eq!(item.metadata.name, "jellysquid3");
        assert_eq!(item.fetched_at, fetched_at);
        assert!(item.is_stale(SystemTime::now()));
    }
}
