use std::{
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use serde::Deserialize;

use crate::{
    features::{
        events::{ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt},
        java::{JavaDomainError, JreProvider},
    },
    libs::request_client::{Request, RequestClient, RequestClientExt, RequestError},
    shared::remove_dir_all,
};

use super::JAVA_WINDOW_BIN;

#[derive(Deserialize, Clone)]
struct Package {
    pub download_url: String,
    pub name: PathBuf,
}

const AZUL_PACKAGES_BASE_API_URL: &str = "https://api.azul.com/metadata/v1/zulu/packages";

const AZUL_PACKAGES_DEFAULT_QUERY_PARAMS: &str =
    "archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1";

pub struct AzulJreProvider<PS: ProgressService, RC: RequestClient> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
}

impl<PS: ProgressService, RC: RequestClient> AzulJreProvider<PS, RC> {
    pub fn new(progress_service: Arc<PS>, request_client: Arc<RC>) -> Self {
        Self {
            progress_service,
            request_client,
        }
    }

    fn build_packages_url(version: u32) -> String {
        format!(
            "{AZUL_PACKAGES_BASE_API_URL}?arch={arch}&java_version={version}&os={os}&{params}",
            arch = std::env::consts::ARCH,
            version = version,
            os = std::env::consts::OS,
            params = AZUL_PACKAGES_DEFAULT_QUERY_PARAMS
        )
    }

    async fn emit_progress(&self, progress_bar_id: Option<&ProgressBarId>, value: f64, msg: &str) {
        if let Some(progress_bar_id) = progress_bar_id {
            self.progress_service
                .emit_progress_safe(progress_bar_id, value, Some(msg))
                .await;
        }
    }

    async fn fetch_package(&self, version: u32) -> Result<Package, JavaDomainError> {
        let packages_url = Self::build_packages_url(version);

        let packages: Vec<Package> = self
            .request_client
            .fetch_json_with_progress(Request::get(packages_url), None)
            .await
            .map_err(|_| get_version_not_available_error(version))?;

        packages
            .first()
            .cloned()
            .ok_or_else(|| get_version_not_available_error(version))
    }

    fn resolve_java_executable_path(path: &Path, package: &Package, _version: u32) -> PathBuf {
        let base_path = path.join(
            package
                .name
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        #[cfg(target_os = "macos")]
        {
            base_path
                .join(format!("zulu-{}.jre", _version))
                .join("Contents")
                .join("Home")
                .join("bin")
                .join("java")
        }

        #[cfg(not(target_os = "macos"))]
        {
            base_path.join("bin").join(JAVA_WINDOW_BIN)
        }
    }

    async fn remove_old_jre_installation(path: &Path) -> Result<(), JavaDomainError> {
        if !path.exists() {
            return Ok(());
        }

        remove_dir_all(&path)
            .await
            .map_err(|_| JavaDomainError::RemoveOldInstallationError {
                path: path.to_path_buf(),
            })
    }

    async fn unpack_jre(
        &self,
        path: &Path,
        file: Bytes,
        package: &Package,
        version: u32,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<PathBuf, JavaDomainError> {
        use zip::ZipArchive;

        let mut archive =
            ZipArchive::new(Cursor::new(file)).map_err(|_| get_failed_to_install(version))?;

        if archive.is_empty() {
            return Err(get_failed_to_install(version));
        }

        if let Some(file) = archive.file_names().next() {
            if let Some(dir) = file.split('/').next() {
                Self::remove_old_jre_installation(&path.join(dir)).await?;
            }
        }

        self.emit_progress(progress_bar_id, 0.0, "Extracting java")
            .await;
        archive
            .extract(path)
            .map_err(|_| get_failed_to_install(version))?;
        self.emit_progress(progress_bar_id, 10.0, "Done extracting java")
            .await;

        Ok(Self::resolve_java_executable_path(path, package, version))
    }

    async fn download_package(
        &self,
        package: &Package,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<Bytes, RequestError> {
        self.request_client
            .fetch_bytes_with_progress(
                Request::get(&package.download_url),
                progress_bar_id.map(|progress_bar_id| (progress_bar_id, 80.0)),
            )
            .await
    }

    async fn download_jre(
        &self,
        version: u32,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<(Package, Bytes), JavaDomainError> {
        self.emit_progress(progress_bar_id, 0.0, "Fetching java version")
            .await;
        let package = self.fetch_package(version).await?;
        self.emit_progress(progress_bar_id, 10.0, "Downloading java version")
            .await;

        let file = self
            .download_package(&package, progress_bar_id)
            .await
            .map_err(|_| get_version_get_failed(version))?;

        Ok((package, file))
    }
}

#[async_trait]
impl<PS: ProgressService, RC: RequestClient> JreProvider for AzulJreProvider<PS, RC> {
    async fn install(&self, version: u32, install_dir: &Path) -> Result<PathBuf, JavaDomainError> {
        let progress_bar_id = self
            .progress_service
            .init_progress_safe(
                ProgressEventType::JavaDownload { version },
                100.0,
                "Downloading java version".to_string(),
            )
            .await;

        let (package, file) = self.download_jre(version, progress_bar_id.as_ref()).await?;

        self.unpack_jre(
            install_dir,
            file,
            &package,
            version,
            progress_bar_id.as_ref(),
        )
        .await
    }
}

fn get_version_not_available_error(version: u32) -> JavaDomainError {
    JavaDomainError::VersionNotAvailable {
        version,
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    }
}

fn get_version_get_failed(version: u32) -> JavaDomainError {
    JavaDomainError::VersionGetFailed {
        version,
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    }
}

fn get_failed_to_install(version: u32) -> JavaDomainError {
    JavaDomainError::FailedToInstall {
        version,
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    }
}
