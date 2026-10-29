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
        events::{
            OptionalProgressBarExt, ProgressBarId, ProgressService, ProgressServiceExt,
            SharedDownload,
        },
        java::{JavaDomainError, JreProvider},
    },
    shared::hash::{FileHash, HashAlgorithm},
    shared::io::infra::remove_dir_all,
    shared::request_client::{Request, RequestClient, RequestClientExt, RequestError},
};

use super::JAVA_WINDOW_BIN;

#[derive(Deserialize, Clone)]
struct Package {
    pub download_url: String,
    pub name: PathBuf,
    /// What the archive is checked against. Required, so an answer without it cannot lead to an
    /// unverified JRE.
    pub sha256_hash: String,
}

const AZUL_PACKAGES_BASE_API_URL: &str = "https://api.azul.com/metadata/v1/zulu/packages";

const AZUL_PACKAGES_DEFAULT_QUERY_PARAMS: &str = "archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1&include_fields=sha256_hash";

pub struct AzulJreProvider<PS: ProgressService, RC: RequestClient> {
    progress_service: Arc<PS>,
    metadata_client: Arc<RC>,
    download_client: Arc<RC>,
}

impl<PS: ProgressService, RC: RequestClient> AzulJreProvider<PS, RC> {
    pub fn new(
        progress_service: Arc<PS>,
        metadata_client: Arc<RC>,
        download_client: Arc<RC>,
    ) -> Self {
        Self {
            progress_service,
            metadata_client,
            download_client,
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
            .metadata_client
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

        // Unpacked beside the install first, and only then put in place of the old one: an
        // archive that fails half-way leaves the JRE that was there.
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|_| get_failed_to_install(version))?;
        let staging = tempfile::Builder::new()
            .prefix(".jre-")
            .tempdir_in(path)
            .map_err(|_| get_failed_to_install(version))?;

        self.emit_progress(progress_bar_id, 0.0, "Extracting java")
            .await;
        archive
            .extract(staging.path())
            .map_err(|_| get_failed_to_install(version))?;
        Self::replace_installed(staging.path(), path, version).await?;
        self.emit_progress(progress_bar_id, 10.0, "Done extracting java")
            .await;

        Ok(Self::resolve_java_executable_path(path, package, version))
    }

    /// Move what `staging` holds into `path`, each entry in place of the one of its name.
    async fn replace_installed(
        staging: &Path,
        path: &Path,
        version: u32,
    ) -> Result<(), JavaDomainError> {
        let mut entries = tokio::fs::read_dir(staging)
            .await
            .map_err(|_| get_failed_to_install(version))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|_| get_failed_to_install(version))?
        {
            let target = path.join(entry.file_name());
            Self::remove_old_jre_installation(&target).await?;
            tokio::fs::rename(entry.path(), &target)
                .await
                .map_err(|_| get_failed_to_install(version))?;
        }

        Ok(())
    }

    async fn download_package(
        &self,
        package: &Package,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<Bytes, RequestError> {
        let hash = FileHash::new(HashAlgorithm::Sha256, package.sha256_hash.as_str())
            .map_err(|err| RequestError::Middleware(err.into()))?;
        let request = Request::get(&package.download_url).with_hash(hash);

        self.download_client
            .fetch_bytes_with_progress(
                request,
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

    async fn download_and_unpack(
        &self,
        version: u32,
        install_dir: &Path,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<PathBuf, JavaDomainError> {
        let (package, file) = self.download_jre(version, progress_bar_id).await?;
        self.unpack_jre(install_dir, file, &package, version, progress_bar_id)
            .await
    }
}

#[async_trait]
impl<PS: ProgressService, RC: RequestClient> JreProvider for AzulJreProvider<PS, RC> {
    async fn install(&self, version: u32, install_dir: &Path) -> Result<PathBuf, JavaDomainError> {
        // Shared: every install needing this Java waits on this one download.
        let progress_bar_id = self
            .progress_service
            .init_shared_progress_safe(
                SharedDownload::Java { version },
                100.0,
                "Downloading java version".to_string(),
            )
            .await;

        let result = self
            .download_and_unpack(version, install_dir, progress_bar_id.as_ref())
            .await;
        progress_bar_id.fail_on_err(&result).await;
        result
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

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Write, sync::Mutex};

    use tokio::sync::mpsc;

    use super::*;
    use crate::{
        features::events::{
            ProgressServiceImpl, infra::InMemoryProgressBarStorage, test_support::ChannelEmitter,
        },
        shared::{
            hash::{HashAlgorithm, infra::hash_hex_async},
            request_client::{BytesProgress, FileDownload},
        },
    };

    const VERSION: u32 = 21;
    const ARCHIVE_URL: &str = "https://cdn.example/zulu-jre.zip";
    /// The package's name, which names the directory the archive holds.
    const PACKAGE_DIR: &str = "zulu-jre";

    /// Serves fixed bodies by URL, and refuses a body that is not the one a request expects.
    #[derive(Default)]
    struct FakeClient(Mutex<HashMap<String, Bytes>>);

    impl FakeClient {
        fn serve(&self, url: &str, body: impl Into<Bytes>) {
            self.0.lock().unwrap().insert(url.to_owned(), body.into());
        }
    }

    #[async_trait]
    impl RequestClient for FakeClient {
        async fn fetch_bytes(&self, request: Request) -> Result<Bytes, RequestError> {
            self.fetch_bytes_with_progress(request, None).await
        }

        async fn fetch_bytes_with_progress(
            &self,
            request: Request,
            _: Option<(&ProgressBarId, f64)>,
        ) -> Result<Bytes, RequestError> {
            let body =
                self.0
                    .lock()
                    .unwrap()
                    .get(&request.url)
                    .cloned()
                    .ok_or(RequestError::Status {
                        status: 404,
                        url: request.url.clone(),
                    })?;
            if let Some(expected) = &request.hash {
                let actual = hash_hex_async(expected.algorithm(), body.clone())
                    .await
                    .unwrap();
                if !expected.matches(&actual) {
                    return Err(RequestError::Hash {
                        actual,
                        expected: expected.hex().to_owned(),
                    });
                }
            }
            Ok(body)
        }

        async fn download_to_file(
            &self,
            _: Request,
            _: &Path,
            _: u64,
            _: Option<BytesProgress<'_>>,
        ) -> Result<u64, RequestError> {
            unimplemented!("the archive is fetched whole")
        }

        async fn download_to_file_conditional(
            &self,
            _: Request,
            _: &Path,
            _: u64,
        ) -> Result<FileDownload, RequestError> {
            unimplemented!("the archive is fetched whole")
        }
    }

    type Provider = AzulJreProvider<ProgressServiceImpl<InMemoryProgressBarStorage>, FakeClient>;

    struct Fixture {
        provider: Provider,
        client: Arc<FakeClient>,
        install_dir: PathBuf,
        _events: mpsc::UnboundedReceiver<crate::features::events::Event>,
        _temp: tempfile::TempDir,
    }

    fn fixture() -> Fixture {
        let temp = tempfile::TempDir::new().unwrap();
        let (tx, events) = mpsc::unbounded_channel();
        let progress = Arc::new(ProgressServiceImpl::new(
            Arc::new(ChannelEmitter(tx)),
            Arc::new(InMemoryProgressBarStorage::default()),
        ));
        let client = Arc::new(FakeClient::default());
        Fixture {
            provider: AzulJreProvider::new(progress, client.clone(), client.clone()),
            client,
            install_dir: temp.path().join("java"),
            _events: events,
            _temp: temp,
        }
    }

    /// The answer of the packages API for [`ARCHIVE_URL`], with `sha256_hash` when given.
    fn serve_package(client: &FakeClient, sha256_hash: Option<&str>) {
        let mut package = serde_json::json!({
            "download_url": ARCHIVE_URL,
            "name": format!("{PACKAGE_DIR}.zip"),
        });
        if let Some(hash) = sha256_hash {
            package["sha256_hash"] = hash.into();
        }
        client.serve(
            &Provider::build_packages_url(VERSION),
            serde_json::to_vec(&vec![package]).unwrap(),
        );
    }

    /// A stored archive of a JRE directory holding `java` with `contents`.
    fn jre_zip(contents: &[u8]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                format!("{PACKAGE_DIR}/bin/java"),
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(contents).unwrap();
        writer.finish().unwrap().into_inner()
    }

    fn installed_java(install_dir: &Path) -> PathBuf {
        install_dir.join(PACKAGE_DIR).join("bin").join("java")
    }

    /// Serve `archive` with a package answer carrying its real digest.
    async fn serve_archive(client: &FakeClient, archive: Vec<u8>) {
        let digest = hash_hex_async(HashAlgorithm::Sha256, Bytes::from(archive.clone()))
            .await
            .unwrap();
        serve_package(client, Some(&digest));
        client.serve(ARCHIVE_URL, archive);
    }

    #[test]
    fn should_request_the_hash_field_when_building_the_packages_url() {
        assert!(Provider::build_packages_url(VERSION).contains("include_fields=sha256_hash"));
    }

    #[tokio::test]
    async fn should_fail_when_the_package_answer_has_no_hash() {
        let fixture = fixture();
        serve_package(&fixture.client, None);
        fixture.client.serve(ARCHIVE_URL, jre_zip(b"new jre"));

        let result = fixture
            .provider
            .install(VERSION, &fixture.install_dir)
            .await;

        assert!(
            matches!(result, Err(JavaDomainError::VersionNotAvailable { .. })),
            "{result:?}"
        );
        assert!(!installed_java(&fixture.install_dir).exists());
    }

    #[tokio::test]
    async fn should_install_the_jre_when_the_archive_unpacks() {
        let fixture = fixture();
        serve_archive(&fixture.client, jre_zip(b"new jre")).await;

        fixture
            .provider
            .install(VERSION, &fixture.install_dir)
            .await
            .unwrap();

        assert_eq!(
            std::fs::read(installed_java(&fixture.install_dir)).unwrap(),
            b"new jre"
        );
        let left: Vec<_> = std::fs::read_dir(&fixture.install_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(left, [PACKAGE_DIR], "no staging directory is left");
    }

    #[tokio::test]
    async fn should_replace_the_installed_jre_when_the_archive_unpacks() {
        let fixture = fixture();
        std::fs::create_dir_all(installed_java(&fixture.install_dir).parent().unwrap()).unwrap();
        std::fs::write(installed_java(&fixture.install_dir), b"old jre").unwrap();
        serve_archive(&fixture.client, jre_zip(b"new jre")).await;

        fixture
            .provider
            .install(VERSION, &fixture.install_dir)
            .await
            .unwrap();

        assert_eq!(
            std::fs::read(installed_java(&fixture.install_dir)).unwrap(),
            b"new jre"
        );
    }

    /// An archive that fails half-way must not leave the installation without a JRE.
    #[tokio::test]
    async fn should_keep_the_installed_jre_when_the_archive_is_corrupt() {
        let fixture = fixture();
        std::fs::create_dir_all(installed_java(&fixture.install_dir).parent().unwrap()).unwrap();
        std::fs::write(installed_java(&fixture.install_dir), b"old jre").unwrap();
        let mut archive = jre_zip(b"new jre contents");
        // A flipped byte of the stored file: the archive opens, the file fails its checksum.
        let at = archive
            .windows(16)
            .position(|window| window == b"new jre contents")
            .unwrap();
        archive[at] ^= 0xff;
        serve_archive(&fixture.client, archive).await;

        let result = fixture
            .provider
            .install(VERSION, &fixture.install_dir)
            .await;

        assert!(
            matches!(result, Err(JavaDomainError::FailedToInstall { .. })),
            "{result:?}"
        );
        assert_eq!(
            std::fs::read(installed_java(&fixture.install_dir)).unwrap(),
            b"old jre"
        );
        let left: Vec<_> = std::fs::read_dir(&fixture.install_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(left, [PACKAGE_DIR], "no staging directory is left");
    }

    #[tokio::test]
    async fn should_refuse_the_archive_when_its_checksum_does_not_match() {
        let fixture = fixture();
        serve_package(&fixture.client, Some(&"0".repeat(64)));
        fixture.client.serve(ARCHIVE_URL, jre_zip(b"new jre"));

        let result = fixture
            .provider
            .install(VERSION, &fixture.install_dir)
            .await;

        assert!(
            matches!(result, Err(JavaDomainError::VersionGetFailed { .. })),
            "{result:?}"
        );
        assert!(!installed_java(&fixture.install_dir).exists());
    }
}
