use std::sync::Arc;

use bytes::Bytes;

use crate::{
    features::{
        events::{ProgressBarId, ProgressService},
        minecraft::{MinecraftDomainError, vanilla},
    },
    shared::request_client::RequestClient,
    shared::{cache::domain::FileStore, cache::infra::InfinityCachedResource, io::domain::IoError},
};

use super::{VersionProgress, checked_get, version_jar_key};

pub struct ClientService<RC: RequestClient, FS: FileStore> {
    request_client: Arc<RC>,
    cached_resource: InfinityCachedResource<FS>,
}

impl<RC: RequestClient, FS: FileStore> ClientService<RC, FS> {
    pub fn new(request_client: Arc<RC>, file_store: Arc<FS>) -> Self {
        Self {
            request_client,
            cached_resource: InfinityCachedResource { cache: file_store },
        }
    }

    fn get_client_download<'a>(
        version_id: &str,
        version_info: &'a vanilla::VersionInfo,
    ) -> Result<&'a vanilla::Download, MinecraftDomainError> {
        version_info
            .downloads
            .get(&vanilla::DownloadType::Client)
            .ok_or(MinecraftDomainError::VersionNotFound {
                version: version_id.to_owned(),
            })
    }

    async fn fetch_client(
        &self,
        version_id: &str,
        version_info: &vanilla::VersionInfo,
        bar: Option<&ProgressBarId>,
    ) -> Result<Bytes, MinecraftDomainError> {
        let client_download = Self::get_client_download(version_id, version_info)?;

        Ok(self
            .request_client
            .fetch_bytes_with_progress(
                checked_get(&client_download.url, &client_download.sha1),
                bar.map(|bar| (bar, 1.0)),
            )
            .await
            .map_err(get_network_error)?)
    }

    /// The size of the client of `version_info` when it is to be downloaded; `None` when it is
    /// there already.
    pub(crate) async fn pending(
        &self,
        version_info: &vanilla::VersionInfo,
        force: bool,
    ) -> Result<Option<u64>, MinecraftDomainError> {
        if !force
            && self
                .cached_resource
                .cache
                .exists(&version_jar_key(version_info.id.clone()))
                .await
        {
            return Ok(None);
        }
        let download = Self::get_client_download(&version_info.id, version_info)?;
        Ok(Some(download.size.into()))
    }

    /// Download the client under a bar of its own in `progress`. The jar appears whole or not
    /// at all ([`FileStore::write`]).
    pub(crate) async fn download_client<PS: ProgressService>(
        &self,
        version_info: &vanilla::VersionInfo,
        force: bool,
        progress: &VersionProgress<'_, PS>,
    ) -> Result<(), MinecraftDomainError> {
        let version_id = &version_info.id;
        let file = progress.start_file(&format!("{version_id}.jar")).await;

        let result = self
            .cached_resource
            .ensure(
                || version_jar_key(version_id.clone()),
                self.fetch_client(version_id, version_info, file.bar()),
                || format!("Client {version_id}"),
                force,
            )
            .await;
        file.settle(&result).await;
        result?;

        if let Ok(download) = Self::get_client_download(version_id, version_info) {
            progress.add_done(download.size.into()).await;
        }
        Ok(())
    }
}

fn get_network_error<E>(error: E) -> IoError
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    IoError::IoError(std::io::Error::new(
        std::io::ErrorKind::NetworkUnreachable,
        error,
    ))
}
