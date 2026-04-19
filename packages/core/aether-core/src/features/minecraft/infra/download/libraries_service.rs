use std::{collections::HashMap, path::PathBuf, sync::Arc};

use bytes::Bytes;
use futures::StreamExt;
use tracing::{debug, error, info, trace, warn};

use crate::{
    features::{
        events::{
            ProgressConfig, ProgressService,
            utils::{ProgressConfigWithMessage, try_for_each_concurrent_with_progress},
        },
        minecraft::{MinecraftDomainError, utils::parse_rules, vanilla},
        settings::LocationInfo,
    },
    libs::request_client::{Request, RequestClient},
    shared::{IoError, create_dir_all, write_async},
};

const MINECRAFT_LIBRARIES_BASE_URL: &str = "https://libraries.minecraft.net/";

pub struct LibrariesService<RC: RequestClient, PS: ProgressService> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
}

impl<RC: RequestClient, PS: ProgressService> LibrariesService<RC, PS> {
    pub fn new(
        progress_service: Arc<PS>,
        request_client: Arc<RC>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            progress_service,
            request_client,
            location_info,
        }
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Bytes, IoError> {
        self.request_client
            .fetch_bytes(Request::get(url))
            .await
            .map_err(get_network_error)
    }

    async fn create_libraries_directories(
        &self,
        version_id: &str,
    ) -> Result<(), MinecraftDomainError> {
        trace!("Creating library directories for version: {}", version_id);

        tokio::try_join! {
            create_dir_all(self.location_info.libraries_dir()),
            create_dir_all(self.location_info.version_natives_dir(version_id)),
        }
        .map_err(|e| {
            error!(
                "Failed to create library directories for version {}: {:?}",
                version_id, e
            );
            e
        })?;

        Ok(())
    }

    pub async fn download_libraries(
        &self,
        libraries: &[vanilla::Library],
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        progress_config: Option<&ProgressConfig<'_>>,
    ) -> Result<(), MinecraftDomainError> {
        info!("Downloading libraries for version: {}", version_info.id);

        self.create_libraries_directories(&version_info.id).await?;

        let progress_config = progress_config
            .as_ref()
            .map(|config| ProgressConfigWithMessage {
                progress_bar_id: config.progress_bar_id,
                total_progress: config.total_progress,
                progress_message: None,
            });

        let libraries_stream = futures::stream::iter(libraries.iter())
            .map(Ok::<&vanilla::Library, MinecraftDomainError>);

        try_for_each_concurrent_with_progress(
            self.progress_service.clone(),
            libraries_stream,
            None,
            libraries.len(),
            progress_config.as_ref(),
            |library| async move {
                self.download_library(library, version_info, java_arch, force, minecraft_updated)
                    .await
            },
        )
        .await?;

        info!(
            "Successfully downloaded libraries for version: {}",
            version_info.id
        );

        Ok(())
    }

    fn should_download_library(
        library: &vanilla::Library,
        java_arch: &str,
        minecraft_updated: bool,
    ) -> bool {
        if let Some(rules) = &library.rules
            && !parse_rules(rules, java_arch, minecraft_updated)
        {
            trace!("Library {} skipped due to rules", library.name);
            return false;
        }

        if !library.downloadable {
            trace!("Library {} is not downloadable", library.name);
            return false;
        }

        true
    }

    pub async fn download_library(
        &self,
        library: &vanilla::Library,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
    ) -> Result<(), MinecraftDomainError> {
        trace!(
            library=%library.name,
            library_data=?library,
            "Processing library"
        );

        if !Self::should_download_library(library, java_arch, minecraft_updated) {
            return Ok(());
        }

        tokio::try_join! {
            self.download_java_library(library, force),
            async {

                if let Err(err) = self.download_native_library_files(library, version_info, java_arch, force).await {
                    warn!("Failed to download native library {}: {:?}", library.name, err);
                }

                Ok(())
            }
        }?;

        Ok(())
    }

    async fn try_download_from_artifact(
        &self,
        library: &vanilla::Library,
        library_path: &PathBuf,
    ) -> Result<(), MinecraftDomainError> {
        if let Some(downloads) = &library.downloads
            && let Some(artifact) = &downloads.artifact
            && !artifact.url.is_empty()
        {
            let bytes = self.fetch_bytes(&artifact.url).await?;
            write_async(library_path, &bytes).await?;
            return Ok(());
        }

        Err(MinecraftDomainError::StorageFailure {
            reason: "No artifact URL found".to_owned(),
        })
    }

    async fn download_from_fallback_url(
        &self,
        library: &vanilla::Library,
        library_path: &PathBuf,
        path_part: &str,
    ) -> Result<(), MinecraftDomainError> {
        let base_url = library
            .url
            .as_deref()
            .unwrap_or(MINECRAFT_LIBRARIES_BASE_URL);

        let url = format!("{base_url}{path_part}");

        let bytes = self.fetch_bytes(&url).await?;
        write_async(library_path, &bytes).await?;

        Ok(())
    }

    pub async fn download_java_library(
        &self,
        library: &vanilla::Library,
        force: bool,
    ) -> Result<(), MinecraftDomainError> {
        let library_path_part = daedalus::get_path_from_artifact(&library.name)?;

        let library_path = self.location_info.libraries_dir().join(&library_path_part);

        if library_path.exists() && !force {
            trace!(
                library = %library.name,
                "Library already exists, skipping"
            );
            return Ok(());
        }

        debug!("Downloading java library \"{}\"", &library.name);

        // Get library by artifact url
        match self
            .try_download_from_artifact(library, &library_path)
            .await
        {
            Ok(()) => return Ok(()),
            Err(err) => warn!(
                "Failed to download {} from artifact, trying fallback: {:?}",
                library.name, err
            ),
        }

        self.download_from_fallback_url(library, &library_path, &library_path_part)
            .await
            .map_err(|e| {
                error!(
                    "Failed to download {} from fallback URL: {:?}",
                    library.name, e
                );
                e
            })?;

        Ok(())
    }

    fn get_native_classifiers<'a>(
        library: &'a vanilla::Library,
        java_arch: &str,
    ) -> Option<(&'a str, &'a HashMap<String, vanilla::LibraryDownload>)> {
        use vanilla::Os;

        let native_os = Os::native_arch(java_arch);
        let natives = library.natives.as_ref()?;
        let os_key = natives.get(&native_os)?;
        let classifiers = library.downloads.as_ref()?.classifiers.as_ref()?;

        Some((os_key, classifiers))
    }

    async fn extract_native_library(
        &self,
        bytes: Bytes,
        version_info: &vanilla::VersionInfo,
    ) -> Result<(), MinecraftDomainError> {
        let reader = std::io::Cursor::new(&bytes);

        let mut archive = zip::ZipArchive::new(reader).map_err(|err| {
            error!("Failed to create zip archive: {:?}", err);
            IoError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        })?;

        let extract_path = self.location_info.version_natives_dir(&version_info.id);

        archive.extract(&extract_path).map_err(|err| {
            error!(
                "Failed to extract zip archive to {:?}: {:?}",
                extract_path, err
            );
            IoError::IoError(std::io::Error::other(err))
        })?;

        Ok(())
    }

    pub async fn download_native_library_files(
        &self,
        library: &vanilla::Library,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        _force: bool,
    ) -> Result<(), MinecraftDomainError> {
        let Some((os_key, classifiers)) = Self::get_native_classifiers(library, java_arch) else {
            trace!("No native classifiers found for library: {}", library.name);
            return Ok(());
        };

        let parsed_key = os_key.replace("${arch}", crate::shared::ARCH_WIDTH);

        let Some(native) = classifiers.get(&parsed_key) else {
            trace!("No native found for key: {}", parsed_key);
            return Ok(());
        };

        debug!("Downloading native library \"{}\"", &library.name);

        let bytes = self.fetch_bytes(&native.url).await.map_err(|e| {
            error!("Failed to fetch native library {}: {:?}", library.name, e);
            e
        })?;

        self.extract_native_library(bytes, version_info)
            .await
            .map_err(|e| {
                error!("Failed to extract native library {}: {:?}", library.name, e);
                e
            })?;

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
