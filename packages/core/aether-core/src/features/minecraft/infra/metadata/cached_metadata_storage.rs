use std::time::Duration;

use async_trait::async_trait;
use daedalus::{minecraft::VersionManifest, modded};

use crate::{
    features::minecraft::{MetadataStorage, MinecraftDomainError, ModLoader},
    shared::{Cache, CachedResource},
};

use super::{loader_manifest_key, version_manifest_key};

const CACHE_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 7);

pub struct CachedMetadataStorage<C: Cache, S: MetadataStorage> {
    cached_resource: CachedResource<C>,
    storage: S,
}

impl<C: Cache, S: MetadataStorage> CachedMetadataStorage<C, S> {
    pub fn new(cache: C, storage: S) -> Self {
        Self {
            cached_resource: CachedResource::new(cache),
            storage,
        }
    }
}

#[async_trait]
impl<C: Cache, S: MetadataStorage> MetadataStorage for CachedMetadataStorage<C, S> {
    async fn get_version_manifest(&self) -> Result<VersionManifest, MinecraftDomainError> {
        self.cached_resource
            .get_cached(
                version_manifest_key,
                self.storage.get_version_manifest(),
                || "version-manifest".to_string(),
                CACHE_TTL,
            )
            .await
    }

    async fn get_loader_version_manifest(
        &self,
        loader: ModLoader,
    ) -> Result<modded::Manifest, MinecraftDomainError> {
        self.cached_resource
            .get_cached(
                || loader_manifest_key(loader),
                self.storage.get_loader_version_manifest(loader),
                || format!("loader manifest {}", loader.as_meta_str()),
                CACHE_TTL,
            )
            .await
    }
}
