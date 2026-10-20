use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::Bytes;
use futures::TryStreamExt;
use tracing::{debug, error, info, trace, warn};

use crate::{
    features::{
        events::{ProgressBarId, ProgressService},
        minecraft::{MinecraftDomainError, parse_rules, vanilla},
        settings::LocationInfo,
    },
    shared::{
        io::{
            domain::IoError,
            infra::{create_dir_all, write_replacing_async},
        },
        keyed_locks::KeyedLocks,
        request_client::{Request, RequestClient},
    },
};

use super::{VersionProgress, checked_get};

const MINECRAFT_LIBRARIES_BASE_URL: &str = "https://libraries.minecraft.net/";

/// Libraries are shared by every game version that lists them; one of them is written by one
/// install at a time ([`KeyedLocks`] by path), and whole or not at all, so another never takes
/// a file still being written for one that is there.
pub struct LibrariesService<RC: RequestClient> {
    request_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
    path_locks: KeyedLocks<PathBuf>,
}

/// A library with something to download: its jar, when it is not there yet, or its natives.
pub(crate) struct PendingLibrary<'a> {
    library: &'a vanilla::Library,
    /// Where its jar goes, under the libraries directory.
    path_part: String,
    jar: bool,
    natives: bool,
    /// What is to be downloaded, as far as the metadata says.
    pub(super) size: u64,
}

impl<RC: RequestClient> LibrariesService<RC> {
    pub fn new(request_client: Arc<RC>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            request_client,
            location_info,
            path_locks: KeyedLocks::default(),
        }
    }

    async fn fetch_bytes(
        &self,
        url: &str,
        sha1: Option<&str>,
        bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<Bytes, IoError> {
        let request = match sha1 {
            Some(sha1) => checked_get(url, sha1),
            // From a repository that names no digest.
            None => Request::get(url).as_download(),
        };
        self.request_client
            .fetch_bytes_with_progress(request, bar)
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

    /// The libraries of `version_info` with something to download for this machine: a jar not
    /// there yet (any, when `force`), or natives, which are extracted on every install. The jars
    /// are looked for on a blocking thread.
    pub(crate) async fn pending<'a>(
        &self,
        version_info: &'a vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
    ) -> Result<Vec<PendingLibrary<'a>>, MinecraftDomainError> {
        let mut candidates = Vec::new();
        for library in &version_info.libraries {
            if Self::should_download_library(library, java_arch, minecraft_updated) {
                candidates.push((library, daedalus::get_path_from_artifact(&library.name)?));
            }
        }
        let missing = if force {
            vec![true; candidates.len()]
        } else {
            let libraries_dir = self.location_info.libraries_dir();
            let paths: Vec<_> = candidates
                .iter()
                .map(|(_, path_part)| libraries_dir.join(path_part))
                .collect();
            tokio::task::spawn_blocking(move || {
                paths.iter().map(|path| !path.exists()).collect::<Vec<_>>()
            })
            .await
            .map_err(|err| IoError::from(std::io::Error::other(err)))?
        };

        let mut pending = Vec::new();
        for ((library, path_part), jar) in candidates.into_iter().zip(missing) {
            let native = Self::native_download(library, java_arch);
            if !jar && native.is_none() {
                continue;
            }

            let jar_size = library
                .downloads
                .as_ref()
                .and_then(|downloads| downloads.artifact.as_ref())
                .filter(|_| jar)
                .map_or(0, |artifact| u64::from(artifact.size));
            let native_size = native.map_or(0, |native| u64::from(native.size));
            pending.push(PendingLibrary {
                library,
                path_part,
                jar,
                natives: native.is_some(),
                size: jar_size + native_size,
            });
        }
        Ok(pending)
    }

    /// Download `libraries`, each under a bar of its own in `progress`.
    pub(crate) async fn download_libraries<PS: ProgressService>(
        &self,
        libraries: &[PendingLibrary<'_>],
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        progress: &VersionProgress<'_, PS>,
    ) -> Result<(), MinecraftDomainError> {
        info!("Downloading libraries for version: {}", version_info.id);

        self.create_libraries_directories(&version_info.id).await?;

        futures::stream::iter(libraries.iter().map(Ok))
            .try_for_each_concurrent(None, |library| {
                self.download_library(library, version_info, java_arch, force, progress)
            })
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

    async fn download_library<PS: ProgressService>(
        &self,
        pending: &PendingLibrary<'_>,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        progress: &VersionProgress<'_, PS>,
    ) -> Result<(), MinecraftDomainError> {
        let library = pending.library;
        trace!(
            library=%library.name,
            library_data=?library,
            "Processing library"
        );

        let file_name = pending
            .path_part
            .rsplit('/')
            .next()
            .unwrap_or(&pending.path_part);
        let file = progress.start_file(file_name).await;

        let result = async {
            tokio::try_join! {
                async {
                    if pending.jar {
                        self.download_java_library(library, &pending.path_part, force, file.bar())
                            .await?;
                    }
                    Ok::<(), MinecraftDomainError>(())
                },
                async {
                    // A failure is the install's: a game without its natives starts and
                    // crashes, with a reason that says nothing of the download.
                    if pending.natives {
                        self.download_native_library_files(
                            library,
                            version_info,
                            java_arch,
                            file.bar(),
                        )
                        .await?;
                    }

                    Ok(())
                }
            }
        }
        .await;
        file.settle(&result).await;
        result?;

        progress.add_done(pending.size).await;
        Ok(())
    }

    async fn try_download_from_artifact(
        &self,
        library: &vanilla::Library,
        library_path: &Path,
        bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        if let Some(downloads) = &library.downloads
            && let Some(artifact) = &downloads.artifact
            && !artifact.url.is_empty()
        {
            let bytes = self
                .fetch_bytes(
                    &artifact.url,
                    Some(&artifact.sha1),
                    bar.map(|bar| (bar, 1.0)),
                )
                .await?;
            write_replacing_async(library_path, bytes).await?;
            return Ok(());
        }

        Err(MinecraftDomainError::StorageFailure {
            reason: "No artifact URL found".to_owned(),
        })
    }

    async fn download_from_fallback_url(
        &self,
        library: &vanilla::Library,
        library_path: &Path,
        path_part: &str,
        bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        let base_url = library
            .url
            .as_deref()
            .unwrap_or(MINECRAFT_LIBRARIES_BASE_URL);

        let url = format!("{base_url}{path_part}");

        let bytes = self
            .fetch_bytes(&url, None, bar.map(|bar| (bar, 1.0)))
            .await?;
        write_replacing_async(library_path, bytes).await?;

        Ok(())
    }

    /// Download the jar of `library` to `path_part` under the libraries directory, unless it is
    /// there (and not `force`d): checked under the lock of the path, as another install may have
    /// put it there meanwhile.
    async fn download_java_library(
        &self,
        library: &vanilla::Library,
        path_part: &str,
        force: bool,
        bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        let library_path = self.location_info.libraries_dir().join(path_part);
        let _lock = self.path_locks.lock(&library_path).await;

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
            .try_download_from_artifact(library, &library_path, bar)
            .await
        {
            Ok(()) => return Ok(()),
            Err(err) => warn!(
                "Failed to download {} from artifact, trying fallback: {:?}",
                library.name, err
            ),
        }

        self.download_from_fallback_url(library, &library_path, path_part, bar)
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

    /// The natives of `library` for this machine, if it has any.
    fn native_download<'a>(
        library: &'a vanilla::Library,
        java_arch: &str,
    ) -> Option<&'a vanilla::LibraryDownload> {
        let Some((os_key, classifiers)) = Self::get_native_classifiers(library, java_arch) else {
            trace!("No native classifiers found for library: {}", library.name);
            return None;
        };

        let parsed_key = os_key.replace("${arch}", crate::core::domain::extensions::ARCH_WIDTH);

        let native = classifiers.get(&parsed_key);
        if native.is_none() {
            trace!("No native found for key: {}", parsed_key);
        }
        native
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

    /// Download the natives of `library` and extract them for the version. Their bytes count
    /// on `bar`; its fraction follows the jar.
    async fn download_native_library_files(
        &self,
        library: &vanilla::Library,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        let Some(native) = Self::native_download(library, java_arch) else {
            return Ok(());
        };

        debug!("Downloading native library \"{}\"", &library.name);

        let bytes = self
            .fetch_bytes(&native.url, Some(&native.sha1), bar.map(|bar| (bar, 0.0)))
            .await
            .map_err(|e| {
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
