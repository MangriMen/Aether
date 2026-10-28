//! Clears the content icons nothing installed shows any more.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use futures::{StreamExt, future::join_all, stream};

use crate::{
    features::instance::{
        ContentIconCache, ContentMetadataCache, InstanceError, InstanceStorage, PackStorage,
        ProviderId,
    },
    shared::housekeeping::Housekeeping,
};

/// How many instances' pack files are read at once at startup.
const MAX_CONCURRENT_PACK_READS: usize = 8;

/// Keeps the icons of the content installed in any instance, as its cached metadata names them,
/// and removes the others.
///
/// Goes by what is installed, not by time: an icon of installed content is shown however old,
/// as offline it is the only one there is. Does nothing if what is installed cannot be read
/// in full, as a partial answer would remove icons still in use.
pub struct ContentIconHousekeeping {
    instance_storage: Arc<dyn InstanceStorage>,
    pack_storage: Arc<dyn PackStorage>,
    metadata_cache: Arc<dyn ContentMetadataCache>,
    icon_cache: Arc<dyn ContentIconCache>,
}

impl ContentIconHousekeeping {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        pack_storage: Arc<dyn PackStorage>,
        metadata_cache: Arc<dyn ContentMetadataCache>,
        icon_cache: Arc<dyn ContentIconCache>,
    ) -> Self {
        Self {
            instance_storage,
            pack_storage,
            metadata_cache,
            icon_cache,
        }
    }

    /// The icon URLs of every content installed in any instance.
    async fn installed_icon_urls(&self) -> Result<HashSet<String>, InstanceError> {
        let mut ids: HashMap<ProviderId, Vec<String>> = HashMap::new();
        let instances = self.instance_storage.list().await?;
        // Concurrent but bounded, in the order of the instances: the first failure in it is the
        // one returned.
        let reads: Vec<_> = instances
            .iter()
            .map(|instance| self.pack_storage.get_pack_files(instance.id()))
            .collect();
        let packs: Vec<_> = stream::iter(reads)
            .buffered(MAX_CONCURRENT_PACK_READS)
            .collect()
            .await;
        for pack in packs {
            for pack_file in pack?.into_values() {
                let Some(provider_id) = pack_file.update_provider_id else {
                    continue;
                };
                if let Some(info) = pack_file.update.as_ref().and_then(|u| u.get(&provider_id))
                    && !info.content_id.is_empty()
                {
                    ids.entry(provider_id)
                        .or_default()
                        .push(info.content_id.clone());
                }
            }
        }

        let cached = join_all(ids.iter().map(|(provider_id, content_ids)| {
            self.metadata_cache.get_many(provider_id, content_ids)
        }))
        .await;
        Ok(cached
            .into_iter()
            .flat_map(HashMap::into_values)
            .filter_map(|cached| cached.metadata.icon_url)
            .collect())
    }
}

#[async_trait]
impl Housekeeping for ContentIconHousekeeping {
    fn what(&self) -> &'static str {
        "content icons of nothing installed"
    }

    async fn run(&self) -> u64 {
        match self.installed_icon_urls().await {
            Ok(urls) => self.icon_cache.retain(&urls).await as u64,
            Err(err) => {
                tracing::debug!("Content icons are kept, the installed content is unknown: {err}");
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Mutex, time::SystemTime};

    use super::*;
    use crate::features::{
        instance::{
            CachedContentAuthor, CachedContentIcon, CachedContentMetadata,
            CachedContentVersionMetadata, ContentAuthor, ContentFileUpdateInfo, ContentIconError,
            ContentMetadata, ContentVersionMetadata, Instance, InstanceBuilder, InstanceId, Pack,
            PackFile,
        },
        minecraft::ModLoader,
    };
    use rstest::rstest;

    fn instance(id: &str) -> Instance {
        InstanceBuilder::new(
            InstanceId::try_from(id).unwrap(),
            id.into(),
            "1.21.1".into(),
            ModLoader::Fabric,
        )
        .build()
    }

    fn provider(id: &str) -> ProviderId {
        ProviderId {
            plugin_id: id.into(),
            capability_id: "content".into(),
        }
    }

    fn pack_file(provider_id: Option<&ProviderId>, content_id: &str) -> PackFile {
        PackFile {
            file_name: format!("{content_id}.jar"),
            name: None,
            hash: String::new(),
            download: None,
            option: None,
            side: None,
            update_provider_id: provider_id.cloned(),
            update: provider_id.map(|p| {
                HashMap::from([(
                    p.clone(),
                    ContentFileUpdateInfo {
                        content_id: content_id.into(),
                        version: "1".into(),
                    },
                )])
            }),
            installed_by: None,
        }
    }

    fn metadata(content_id: &str, icon_url: Option<&str>) -> ContentMetadata {
        ContentMetadata {
            content_id: content_id.into(),
            slug: content_id.into(),
            name: content_id.into(),
            description: None,
            author: None,
            author_ref: None,
            icon_url: icon_url.map(Into::into),
        }
    }

    /// `None` fails to list.
    struct FakeInstances(Option<Vec<Instance>>);

    #[async_trait]
    impl InstanceStorage for FakeInstances {
        async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
            self.0.clone().ok_or_else(|| InstanceError::NotFound {
                instance_id: "list".into(),
            })
        }

        async fn get(&self, _: &InstanceId) -> Result<Instance, InstanceError> {
            unimplemented!()
        }

        async fn upsert(&self, _: &Instance) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn update(
            &self,
            id: &InstanceId,
            change: crate::features::instance::InstanceChange<'_>,
        ) -> Result<Instance, InstanceError> {
            let mut instance = self.get(id).await?;
            if change(&mut instance)? {
                self.upsert(&instance).await?;
            }
            Ok(instance)
        }

        async fn remove(&self, _: &InstanceId) -> Result<(), InstanceError> {
            unimplemented!()
        }
    }

    /// Files by instance id; an instance without an entry fails to read.
    struct FakePacks(HashMap<String, Vec<PackFile>>);

    #[async_trait]
    impl PackStorage for FakePacks {
        async fn get_pack(&self, _: &InstanceId) -> Result<Pack, InstanceError> {
            unimplemented!()
        }

        async fn update_pack(&self, _: &InstanceId, _: &Pack) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn get_pack_file(&self, _: &InstanceId, _: &str) -> Result<PackFile, InstanceError> {
            unimplemented!()
        }

        async fn get_pack_files(
            &self,
            instance_id: &InstanceId,
        ) -> Result<HashMap<String, PackFile>, InstanceError> {
            let files =
                self.0
                    .get(instance_id.as_str())
                    .ok_or_else(|| InstanceError::NotFound {
                        instance_id: instance_id.to_string(),
                    })?;
            Ok(files
                .iter()
                .map(|f| (f.file_name.clone(), f.clone()))
                .collect())
        }

        async fn find_by_provider_id(
            &self,
            _: &InstanceId,
            _: &ProviderId,
            _: &str,
        ) -> Result<Option<PackFile>, InstanceError> {
            unimplemented!()
        }

        async fn update_pack_file(
            &self,
            _: &InstanceId,
            _: &str,
            _: &PackFile,
        ) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn update_pack_file_many(
            &self,
            _: &InstanceId,
            _: &[String],
            _: &[PackFile],
        ) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn remove_pack_file(&self, _: &InstanceId, _: &str) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn remove_pack_file_many(
            &self,
            _: &InstanceId,
            _: &[String],
        ) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn remove_all_for_instance(&self, _: &InstanceId) -> Result<(), InstanceError> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct FakeMetadata(HashMap<(ProviderId, String), ContentMetadata>);

    #[async_trait]
    impl ContentMetadataCache for FakeMetadata {
        async fn get_many(
            &self,
            provider_id: &ProviderId,
            content_ids: &[String],
        ) -> HashMap<String, CachedContentMetadata> {
            content_ids
                .iter()
                .filter_map(|id| {
                    let metadata = self.0.get(&(provider_id.clone(), id.clone()))?;
                    Some((
                        id.clone(),
                        CachedContentMetadata {
                            metadata: metadata.clone(),
                            fetched_at: SystemTime::now(),
                        },
                    ))
                })
                .collect()
        }

        async fn put_keyed_at(
            &self,
            _: &ProviderId,
            _: &[(String, ContentMetadata)],
            _: SystemTime,
        ) -> bool {
            unimplemented!()
        }

        async fn get_versions(
            &self,
            _: &ProviderId,
            _: &[String],
        ) -> HashMap<String, CachedContentVersionMetadata> {
            unimplemented!()
        }

        async fn put_versions(&self, _: &ProviderId, _: &[ContentVersionMetadata]) -> bool {
            unimplemented!()
        }

        async fn get_authors(
            &self,
            _: &ProviderId,
            _: &[String],
        ) -> HashMap<String, CachedContentAuthor> {
            unimplemented!()
        }

        async fn put_authors(&self, _: &ProviderId, _: &[ContentAuthor]) -> bool {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct FakeIcons {
        retained: Mutex<Option<HashSet<String>>>,
    }

    #[async_trait]
    impl ContentIconCache for FakeIcons {
        async fn lookup(&self, _: &str) -> CachedContentIcon {
            unimplemented!()
        }

        async fn fetch(&self, _: &str) -> Result<bool, ContentIconError> {
            unimplemented!()
        }

        async fn retain(&self, keep: &HashSet<String>) -> usize {
            *self.retained.lock().unwrap() = Some(keep.clone());
            7
        }
    }

    fn housekeeping(
        instances: Option<Vec<Instance>>,
        packs: HashMap<String, Vec<PackFile>>,
        metadata: FakeMetadata,
    ) -> (ContentIconHousekeeping, Arc<FakeIcons>) {
        let icons = Arc::new(FakeIcons::default());
        let housekeeping = ContentIconHousekeeping::new(
            Arc::new(FakeInstances(instances)),
            Arc::new(FakePacks(packs)),
            Arc::new(metadata),
            icons.clone(),
        );
        (housekeeping, icons)
    }

    #[tokio::test]
    async fn should_retain_only_icons_of_installed_content_when_run() {
        let (a, b) = (provider("a"), provider("b"));
        let mut cache = FakeMetadata::default();
        for (p, id, icon) in [
            (&a, "sodium", Some("https://x/sodium.png")),
            (&b, "lithium", Some("https://x/lithium.png")),
            (&a, "uninstalled", Some("https://x/uninstalled.png")),
            (&a, "no-icon", None),
            (&a, "", Some("https://x/empty-id.png")),
        ] {
            cache.0.insert((p.clone(), id.into()), metadata(id, icon));
        }
        let packs = HashMap::from([
            (
                "one".to_string(),
                vec![
                    pack_file(Some(&a), "sodium"),
                    pack_file(None, "manual"),
                    pack_file(Some(&a), ""),
                    pack_file(Some(&a), "no-icon"),
                ],
            ),
            ("two".to_string(), vec![pack_file(Some(&b), "lithium")]),
        ]);
        let (housekeeping, icons) =
            housekeeping(Some(vec![instance("one"), instance("two")]), packs, cache);

        let removed = housekeeping.run().await;

        assert_eq!(removed, 7);
        assert_eq!(
            icons.retained.lock().unwrap().clone(),
            Some(HashSet::from([
                "https://x/sodium.png".to_string(),
                "https://x/lithium.png".to_string(),
            ]))
        );
    }

    #[rstest]
    #[case::instances_cannot_be_listed(None, HashMap::new())]
    #[case::pack_files_cannot_be_read(
        Some(vec![instance("one"), instance("broken")]),
        HashMap::from([("one".to_string(), vec![pack_file(Some(&provider("a")), "sodium")])]),
    )]
    #[tokio::test]
    async fn should_not_retain_when_the_installed_content_is_unknown(
        #[case] instances: Option<Vec<Instance>>,
        #[case] packs: HashMap<String, Vec<PackFile>>,
    ) {
        let (housekeeping, icons) = housekeeping(instances, packs, FakeMetadata::default());

        assert_eq!(housekeeping.run().await, 0);
        assert!(icons.retained.lock().unwrap().is_none());
    }
}
