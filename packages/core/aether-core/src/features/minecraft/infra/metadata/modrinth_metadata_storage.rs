use std::sync::Arc;

use async_trait::async_trait;
use daedalus::{
    minecraft::VersionManifest,
    modded::{self},
};

use crate::{
    features::minecraft::{MinecraftDomainError, ModLoader, MetadataStorage},
    libs::request_client::{Request, RequestClient, RequestClientExt},
    shared::IoError,
};

pub const META_URL: &str = "https://launcher-meta.modrinth.com/";

pub struct ModrinthMetadataStorage<RC: RequestClient> {
    request_client: Arc<RC>,
}

impl<RC: RequestClient> ModrinthMetadataStorage<RC> {
    pub fn new(request_client: Arc<RC>) -> Self {
        Self { request_client }
    }

    fn get_loader_manifest_url(loader: ModLoader) -> String {
        format!("{META_URL}{}/v0/manifest.json", loader.as_meta_str())
    }
}

#[async_trait]
impl<RC: RequestClient> MetadataStorage for ModrinthMetadataStorage<RC> {
    async fn get_version_manifest(&self) -> Result<VersionManifest, MinecraftDomainError> {
        Ok(self
            .request_client
            .fetch_json(Request::get(daedalus::minecraft::VERSION_MANIFEST_URL))
            .await
            .map_err(|err| {
                IoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })?)
    }

    async fn get_loader_version_manifest(
        &self,
        loader: ModLoader,
    ) -> Result<modded::Manifest, MinecraftDomainError> {
        Ok(self
            .request_client
            .fetch_json(Request::get(Self::get_loader_manifest_url(loader)))
            .await
            .map_err(|err| {
                IoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })?)
    }
}
