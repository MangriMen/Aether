use std::sync::Arc;

use tracing::debug;

use crate::{
    features::{
        events::ProgressService,
        minecraft::{MinecraftDomainError, vanilla},
        settings::LocationInfo,
    },
    shared::{
        io::{
            domain::IoError,
            infra::{create_dir_all, write_replacing_async},
        },
        request_client::RequestClient,
    },
};

use super::{VersionProgress, checked_get};

/// The log config a version names (`logging.client`): the game is started with it, which is
/// what switches off the log4j lookups of versions 1.7 to 1.18.0 (`Log4Shell`). Shared by the
/// versions that name the same file.
pub struct LoggingService<RC: RequestClient> {
    request_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
}

impl<RC: RequestClient> LoggingService<RC> {
    pub fn new(request_client: Arc<RC>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            request_client,
            location_info,
        }
    }

    /// The log config of `version_info` that is to be downloaded: `None` when the version names
    /// none, or it is there already (and not `force`d).
    pub(crate) async fn pending<'a>(
        &self,
        version_info: &'a vanilla::VersionInfo,
        force: bool,
    ) -> Option<&'a vanilla::LogConfigDownload> {
        let (_, file) = version_info.client_log_config()?;
        if force {
            return Some(file);
        }

        // An id that is no file name is refused by the download, with a reason.
        let present = match self.location_info.log_config_path(&file.id) {
            Some(path) => tokio::fs::try_exists(path).await.unwrap_or(false),
            None => false,
        };
        (!present).then_some(file)
    }

    /// Download `file` under a bar of its own in `progress`, checked against its hash. The file
    /// appears whole or not at all.
    pub(crate) async fn download_log_config<PS: ProgressService>(
        &self,
        file: &vanilla::LogConfigDownload,
        progress: &VersionProgress<'_, PS>,
    ) -> Result<(), MinecraftDomainError> {
        let path = self
            .location_info
            .log_config_path(&file.id)
            .ok_or_else(|| MinecraftDomainError::StorageFailure {
                reason: format!("Invalid log config name: {}", file.id),
            })?;
        debug!("Downloading log config \"{}\"", file.id);

        create_dir_all(self.location_info.log_configs_dir()).await?;

        let bar = progress.start_file(&file.id).await;
        let result = async {
            let bytes = self
                .request_client
                .fetch_bytes_with_progress(
                    checked_get(&file.url, &file.sha1),
                    bar.bar().map(|bar| (bar, 1.0)),
                )
                .await
                .map_err(|err| {
                    IoError::IoError(std::io::Error::new(
                        std::io::ErrorKind::NetworkUnreachable,
                        err,
                    ))
                })?;
            write_replacing_async(&path, bytes).await?;
            Ok::<(), MinecraftDomainError>(())
        }
        .await;
        bar.settle(&result).await;
        result?;

        progress.add_done(file.size.into()).await;
        Ok(())
    }
}
