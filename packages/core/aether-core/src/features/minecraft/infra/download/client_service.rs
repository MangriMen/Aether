use std::sync::Arc;

use bytes::Bytes;

use crate::{
    features::{
        events::{ProgressBarId, ProgressService, ProgressServiceExt},
        minecraft::MinecraftDomainError,
    },
    libs::request_client::{Request, RequestClient},
    shared::{FileStore, InfinityCachedResource, IoError},
};

use super::version_jar_key;

pub struct ClientService<RC: RequestClient, PS: ProgressService, FS: FileStore> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
    cached_resource: InfinityCachedResource<FS>,
}

impl<RC: RequestClient, PS: ProgressService, FS: FileStore> ClientService<RC, PS, FS> {
    pub fn new(progress_service: Arc<PS>, request_client: Arc<RC>, file_store: Arc<FS>) -> Self {
        Self {
            progress_service,
            request_client,
            cached_resource: InfinityCachedResource {
                cache: file_store.clone(),
            },
        }
    }

    fn get_client_download<'a>(
        version_id: &str,
        version_info: &'a daedalus::minecraft::VersionInfo,
    ) -> Result<&'a daedalus::minecraft::Download, MinecraftDomainError> {
        version_info
            .downloads
            .get(&daedalus::minecraft::DownloadType::Client)
            .ok_or(MinecraftDomainError::VersionNotFound {
                version: version_id.to_owned(),
            })
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Bytes, IoError> {
        self.request_client
            .fetch_bytes(Request::get(url))
            .await
            .map_err(get_network_error)
    }

    async fn fetch_client(
        &self,
        version_id: &str,
        version_info: &daedalus::minecraft::VersionInfo,
    ) -> Result<Bytes, MinecraftDomainError> {
        let client_download_url = Self::get_client_download(version_id, version_info)?;

        Ok(self.fetch_bytes(&client_download_url.url).await?)
    }

    pub async fn download_client(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        force: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        let version_id = &version_info.id;

        self.cached_resource
            .ensure(
                || version_jar_key(version_id.to_string()),
                self.fetch_client(version_id, version_info),
                || format!("Client {version_id}"),
                force,
            )
            .await?;

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(loading_bar, 9.0, None)
                .await;
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
