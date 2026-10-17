use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::minecraft::{MetadataStorage, MinecraftDomainError, ModLoader, modded, vanilla},
    shared::io::domain::IoError,
    shared::request_client::{Request, RequestClient, RequestClientExt},
};

pub(super) const META_URL: &str = "https://launcher-meta.modrinth.com/";

pub struct ModrinthMetadataStorage<RC: RequestClient> {
    request_client: Arc<RC>,
}

fn get_loader_manifest_url(loader: ModLoader) -> String {
    let metadata = daedalus::modded::loader_manifest_metadata(loader.as_meta_str());
    format!("{META_URL}{}", metadata.path)
}

impl<RC: RequestClient> ModrinthMetadataStorage<RC> {
    pub fn new(request_client: Arc<RC>) -> Self {
        Self { request_client }
    }
}

#[async_trait]
impl<RC: RequestClient> MetadataStorage for ModrinthMetadataStorage<RC> {
    async fn get_version_manifest(&self) -> Result<vanilla::VersionManifest, MinecraftDomainError> {
        Ok(self
            .request_client
            .fetch_json::<daedalus::minecraft::VersionManifest>(Request::get(
                daedalus::minecraft::VERSION_MANIFEST_URL,
            ))
            .await
            .map_err(|err| {
                IoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })?
            .into())
    }

    async fn get_loader_version_manifest(
        &self,
        loader: ModLoader,
    ) -> Result<modded::Manifest, MinecraftDomainError> {
        Ok(self
            .request_client
            .fetch_json::<daedalus::modded::Manifest>(Request::get(get_loader_manifest_url(loader)))
            .await
            .map_err(|err| {
                IoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })?
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_request_the_current_manifest_format_of_each_loader() {
        assert_eq!(
            get_loader_manifest_url(ModLoader::Fabric),
            format!("{META_URL}fabric/v0/manifest.json")
        );
        assert_eq!(
            get_loader_manifest_url(ModLoader::Quilt),
            format!("{META_URL}quilt/v1/manifest.json")
        );
        assert_eq!(
            get_loader_manifest_url(ModLoader::NeoForge),
            format!("{META_URL}neo/v0/manifest.json")
        );
    }
}
