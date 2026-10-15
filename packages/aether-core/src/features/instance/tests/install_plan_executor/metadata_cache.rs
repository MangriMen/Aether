//! What a plan's entries say about their own content is cached under the provider the content
//! list will look it up by: the one chosen for the identity's platform.

use std::time::SystemTime;

use crate::features::instance::{
    CachedContentMetadata, ContentMetadata, ContentVersionMetadata, InstallFileEntry,
};

use super::common::*;

const URL: &str = "https://cdn.modrinth.com/data/AANobbMI/versions/v2/sodium-2.jar";

/// A direct entry for sodium `v2` on `platform`, with what its provider says about it.
fn described(platform: &str, content_id: &str, version: &str) -> InstallFileEntry {
    InstallFileEntry {
        metadata: Some(ContentMetadata {
            content_id: content_id.into(),
            slug: "sodium".into(),
            name: "Sodium".into(),
            description: None,
            author: None,
            icon_url: None,
        }),
        version_metadata: Some(ContentVersionMetadata {
            version_id: version.into(),
            version_number: "0.6.0".into(),
            name: None,
        }),
        ..identified(URL, "mods/sodium-2.jar", platform, "AANobbMI", "v2")
    }
}

/// The plan comes from another provider than the one serving the platform: the content list
/// looks the file up by the latter, which is what its update info is recorded under.
#[tokio::test]
async fn should_cache_metadata_under_the_provider_of_the_platform_when_another_provider_installs() {
    let fixture = Fixture::new().await;
    let modrinth = MockProvider::builtin_modrinth();
    fixture.register(&modrinth).await;
    let installing = plugin("packwiz");

    fixture
        .execute(
            existing(vec![described("modrinth", "AANobbMI", "v2")]),
            &installing,
        )
        .await
        .unwrap();

    let items = fixture.metadata_cache.items.lock().unwrap();
    assert_eq!(items.len(), 1);
    // Provisional: stale at once, so the next list refreshes it.
    let project = &items[&(modrinth_id(), "AANobbMI".to_owned())];
    assert_eq!(project.metadata.name, "Sodium");
    assert_eq!(project.fetched_at, SystemTime::UNIX_EPOCH);
    let versions = fixture.metadata_cache.versions.lock().unwrap();
    assert_eq!(versions.len(), 1);
    // Complete: fresh.
    let version = &versions[&(modrinth_id(), "v2".to_owned())];
    assert_eq!(version.metadata.version_number, "0.6.0");
    assert!(version.fetched_at > SystemTime::UNIX_EPOCH);
}

/// What is cached may be complete (with its author): the plan's provisional copy does not
/// replace it.
#[tokio::test]
async fn should_keep_the_cached_project_when_one_is_already_there() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let fetched_at = SystemTime::now();
    let cached = CachedContentMetadata {
        metadata: ContentMetadata {
            author: Some("jellysquid3".into()),
            ..described("modrinth", "AANobbMI", "v2").metadata.unwrap()
        },
        fetched_at,
    };
    fixture
        .metadata_cache
        .items
        .lock()
        .unwrap()
        .insert((modrinth_id(), "AANobbMI".to_owned()), cached.clone());

    fixture
        .execute(
            existing(vec![described("modrinth", "AANobbMI", "v2")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    let items = fixture.metadata_cache.items.lock().unwrap();
    assert_eq!(items[&(modrinth_id(), "AANobbMI".to_owned())], cached);
    // The version still goes in.
    assert_eq!(fixture.metadata_cache.versions.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn should_not_cache_metadata_when_it_is_of_another_content_than_the_identity() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;

    fixture
        .execute(
            existing(vec![described("modrinth", "OTHER", "v1")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert!(fixture.metadata_cache.items.lock().unwrap().is_empty());
    assert!(fixture.metadata_cache.versions.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_not_cache_metadata_when_no_provider_serves_the_platform() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;

    fixture
        .execute(
            existing(vec![described("curseforge", "AANobbMI", "v2")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert!(fixture.metadata_cache.items.lock().unwrap().is_empty());
    assert!(fixture.metadata_cache.versions.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_not_cache_metadata_when_the_entry_is_a_provider_ref() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let entry = InstallFileEntry {
        metadata: described("modrinth", "AANobbMI", "v2").metadata,
        ..provider_ref("modrinth", "AANobbMI", "mods/sodium.jar")
    };

    fixture
        .execute(existing(vec![entry]), &plugin("packwiz"))
        .await
        .unwrap();

    assert!(fixture.metadata_cache.items.lock().unwrap().is_empty());
}
