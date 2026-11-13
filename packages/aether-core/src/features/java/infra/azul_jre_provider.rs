use std::{
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};
use tempfile::TempDir;

use async_trait::async_trait;
use bytes::Bytes;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::{
    features::{
        events::{
            CancellableBar, ProgressBarId, ProgressService, ProgressServiceExt, SharedDownload,
        },
        java::{JavaDomainError, JreProvider},
    },
    shared::hash::{FileHash, HashAlgorithm},
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

    async fn unpack_jre(
        &self,
        path: &Path,
        file: Bytes,
        package: &Package,
        version: u32,
        progress_bar_id: Option<&ProgressBarId>,
        cancel: &CancellationToken,
    ) -> Result<PathBuf, JavaDomainError> {
        // Unpacked beside the install first, and only then put in place of the old one: an
        // archive that fails half-way leaves the JRE that was there.
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|err| get_failed_to_install(version, err))?;
        let staging = tempfile::Builder::new()
            .prefix(STAGING_PREFIX)
            .tempdir_in(path)
            .map_err(|err| get_failed_to_install(version, err))?;

        self.emit_progress(progress_bar_id, 0.0, "Extracting java")
            .await;
        let staging = Self::extract(file, staging)
            .await
            .map_err(|err| get_failed_to_install(version, err))?;
        // A future dropped while it waits here deletes the staging on the runtime: rare, and
        // nothing is lost by it.
        if let Err(err) = self.pass_point_of_no_return(progress_bar_id, cancel).await {
            Self::discard(staging).await;
            return Err(err);
        }
        Self::replace_installed(staging, path.to_path_buf(), version).await?;
        self.emit_progress(progress_bar_id, 10.0, "Done extracting java")
            .await;

        Ok(Self::resolve_java_executable_path(path, package, version))
    }

    /// Replacing the installed JRE entry by entry cannot stop halfway: it would leave neither the
    /// old JRE nor the new one. A call-off that came before this closes the bar is the last one
    /// that is taken (the staging goes with the early return).
    async fn pass_point_of_no_return(
        &self,
        progress_bar_id: Option<&ProgressBarId>,
        cancel: &CancellationToken,
    ) -> Result<(), JavaDomainError> {
        if let Some(bar) = progress_bar_id {
            self.progress_service.close_cancellation_safe(bar).await;
        }
        if cancel.is_cancelled() {
            return Err(JavaDomainError::Cancelled);
        }
        Ok(())
    }

    /// Off the runtime: unzipping a JRE takes seconds of blocking disk work. The blocking task
    /// owns the staging directory, so a future dropped meanwhile (the install was called off)
    /// does not delete it under the extraction: the task ends first and removes it itself.
    async fn extract(file: Bytes, staging: TempDir) -> Result<TempDir, std::io::Error> {
        tokio::task::spawn_blocking(move || {
            let mut archive = zip::ZipArchive::new(Cursor::new(file))?;
            if archive.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "the archive is empty",
                ));
            }
            archive.extract(staging.path())?;
            Ok(staging)
        })
        .await
        .map_err(std::io::Error::other)?
    }

    /// Deleting a half-extracted JRE is many file operations: not on the runtime.
    async fn discard(staging: TempDir) {
        let _ = tokio::task::spawn_blocking(move || drop(staging)).await;
    }

    /// The swap is one blocking task that nothing can drop halfway: a future dropped meanwhile
    /// (the install was called off) leaves it running to the end, so the installation is never
    /// left with neither the old JRE nor the new one.
    async fn replace_installed(
        staging: TempDir,
        path: PathBuf,
        version: u32,
    ) -> Result<(), JavaDomainError> {
        Self::replace_installed_with(staging, path, version, |from, to| std::fs::rename(from, to))
            .await
    }

    async fn replace_installed_with(
        staging: TempDir,
        path: PathBuf,
        version: u32,
        rename: impl Fn(&Path, &Path) -> std::io::Result<()> + Send + 'static,
    ) -> Result<(), JavaDomainError> {
        tokio::task::spawn_blocking(move || swap_in(staging.path(), &path, version, &rename))
            .await
            .map_err(|err| get_failed_to_install(version, std::io::Error::other(err)))?
    }

    async fn download_package(
        &self,
        package: &Package,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<Bytes, RequestError> {
        let hash = FileHash::new(HashAlgorithm::Sha256, package.sha256_hash.as_str())
            .map_err(|err| RequestError::from(anyhow::Error::new(err)))?;
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
        cancel: &CancellationToken,
    ) -> Result<PathBuf, JavaDomainError> {
        // Dropped halfway on a call-off: nothing is written to the install before the unpacking.
        // One that comes as the download ends is decided at the point of no return.
        let (package, file) = tokio::select! {
            biased;
            downloaded = self.download_jre(version, progress_bar_id) => downloaded?,
            () = cancel.cancelled() => return Err(JavaDomainError::Cancelled),
        };
        self.unpack_jre(
            install_dir,
            file,
            &package,
            version,
            progress_bar_id,
            cancel,
        )
        .await
    }
}

#[async_trait]
impl<PS: ProgressService, RC: RequestClient> JreProvider for AzulJreProvider<PS, RC> {
    async fn install(
        &self,
        version: u32,
        install_dir: &Path,
        cancellable: bool,
    ) -> Result<PathBuf, JavaDomainError> {
        // Shared: every install needing this Java waits on this one download.
        // Called off (the future dropped) it ends cancelled, not done.
        let download = SharedDownload::Java { version };
        let message = "Downloading java version".to_string();
        let (bar, cancel) = if cancellable {
            self.progress_service
                .init_cancellable_shared_progress_safe(download, 100.0, message)
                .await
        } else {
            (
                self.progress_service
                    .init_shared_progress_safe(download, 100.0, message)
                    .await,
                CancellationToken::new(),
            )
        };
        let bar = CancellableBar::new(bar);

        let result = self
            .download_and_unpack(version, install_dir, bar.bar(), &cancel)
            .await;
        match &result {
            Err(JavaDomainError::Cancelled) => bar.cancel().await,
            _ => bar.settle(&result).await,
        }
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

const STAGING_PREFIX: &str = ".jre-new-";
const ASIDE_PREFIX: &str = ".jre-old-";
/// Inside the set-aside directory, apart from the previous entries: the names cannot meet.
const DISCARDED_DIR: &str = ".discarded";

type Rename<'a> = &'a dyn Fn(&Path, &Path) -> std::io::Result<()>;

/// An entry of the installation that was set aside for the new one.
struct Moved {
    target: PathBuf,
    /// Where the previous entry waits; `None` when there was none.
    set_aside: Option<PathBuf>,
}

/// Move what `staging` holds into `path`, each entry in place of the one of its name.
/// What was there is set aside first and put back if any step fails: a JRE the game holds
/// open can refuse to move, and the installation must not be left with neither version.
fn swap_in(
    staging: &Path,
    path: &Path,
    version: u32,
    rename: Rename<'_>,
) -> Result<(), JavaDomainError> {
    let aside = tempfile::Builder::new()
        .prefix(ASIDE_PREFIX)
        .tempdir_in(path)
        .map_err(|err| get_failed_to_install(version, err))?;

    let mut moved = Vec::new();
    let result = move_entries(staging, path, aside.path(), &mut moved, version, rename);
    let put_back = result.is_ok() || roll_back(&moved, aside.path(), rename);

    if put_back {
        // Nothing but a leftover can fail here; no later install removes it.
        if let Err(err) = aside.close() {
            tracing::warn!("Could not remove the set-aside JRE directory: {err}");
        }
    } else {
        // The previous entries that could not be put back live on only in here.
        let kept = aside.keep();
        tracing::error!("The previous JRE entries are kept in {kept:?}");
    }
    result
}

fn move_entries(
    staging: &Path,
    path: &Path,
    aside: &Path,
    moved: &mut Vec<Moved>,
    version: u32,
    rename: Rename<'_>,
) -> Result<(), JavaDomainError> {
    for entry in std::fs::read_dir(staging).map_err(|err| get_failed_to_install(version, err))? {
        let entry = entry.map_err(|err| get_failed_to_install(version, err))?;
        let target = path.join(entry.file_name());
        let set_aside = aside.join(entry.file_name());

        // Not following a link: one that dangles is still an entry of the installation.
        let had_old = exists(&target).map_err(|err| get_failed_to_install(version, err))?;
        if had_old {
            rename(&target, &set_aside).map_err(|err| {
                JavaDomainError::RemoveOldInstallationError {
                    path: target.clone(),
                    source: Arc::new(err),
                }
            })?;
        }
        moved.push(Moved {
            target: target.clone(),
            set_aside: had_old.then_some(set_aside),
        });

        rename(&entry.path(), &target).map_err(|err| get_failed_to_install(version, err))?;
    }

    Ok(())
}

/// Best effort; `false` when a previous entry could not be put back.
fn roll_back(moved: &[Moved], aside: &Path, rename: Rename<'_>) -> bool {
    let mut all_back = true;
    for step in moved.iter().rev() {
        let target_exists = exists(&step.target).unwrap_or(true);
        let Some(set_aside) = &step.set_aside else {
            // New where nothing was before: removing it loses no JRE.
            if target_exists && let Err(err) = remove_entry(&step.target) {
                tracing::error!(
                    "Could not remove the new JRE entry {:?}: {err}",
                    step.target
                );
            }
            continue;
        };

        // Moved out of the way, not deleted: a deletion that fails halfway would leave the
        // target occupied and the previous entry unable to return.
        let mut discarded = None;
        if target_exists {
            let name = step.target.file_name().unwrap_or_default();
            let place = aside.join(DISCARDED_DIR).join(name);
            let moved_away = std::fs::create_dir_all(aside.join(DISCARDED_DIR))
                .and_then(|()| rename(&step.target, &place));
            if let Err(err) = moved_away {
                tracing::error!(
                    "Could not move the new JRE entry {:?} out of the way: {err}",
                    step.target
                );
                all_back = false;
                continue;
            }
            discarded = Some(place);
        }
        if let Err(err) = rename(set_aside, &step.target) {
            tracing::error!(
                "Could not put back the previous JRE entry {:?}: {err}",
                step.target
            );
            all_back = false;
            // At least one JRE stays where the game looks for it.
            if let Some(place) = discarded
                && let Err(err) = rename(&place, &step.target)
            {
                tracing::error!(
                    "Could not restore the new JRE entry {:?}: {err}",
                    step.target
                );
            }
        }
    }
    all_back
}

fn exists(path: &Path) -> std::io::Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn remove_entry(path: &Path) -> std::io::Result<()> {
    if std::fs::symlink_metadata(path)?.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}

fn get_failed_to_install(version: u32, source: impl Into<std::io::Error>) -> JavaDomainError {
    JavaDomainError::FailedToInstall {
        version,
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        source: Arc::new(source.into()),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::Write,
        sync::{
            Mutex,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };

    use tokio::sync::{Notify, mpsc};

    use super::*;
    use crate::{
        features::events::{
            CancelState, ProgressBar, ProgressBarStorage, ProgressServiceImpl, ProgressStatus,
            infra::InMemoryProgressBarStorage,
            test_support::{ChannelEmitter, sent},
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
    struct FakeClient {
        bodies: Mutex<HashMap<String, Bytes>>,
        /// Holds the archive back until `proceed`, telling `at_archive` it was asked for.
        hold_archive: AtomicBool,
        at_archive: Notify,
        proceed: Notify,
    }

    impl FakeClient {
        fn serve(&self, url: &str, body: impl Into<Bytes>) {
            self.bodies
                .lock()
                .unwrap()
                .insert(url.to_owned(), body.into());
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
            if request.url == ARCHIVE_URL && self.hold_archive.load(Ordering::SeqCst) {
                self.at_archive.notify_one();
                self.proceed.notified().await;
            }
            let body = self
                .bodies
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
        provider: Arc<Provider>,
        client: Arc<FakeClient>,
        progress: Arc<ProgressServiceImpl<InMemoryProgressBarStorage>>,
        bars: Arc<InMemoryProgressBarStorage>,
        install_dir: PathBuf,
        events: mpsc::UnboundedReceiver<crate::features::events::Event>,
        _temp: tempfile::TempDir,
    }

    fn fixture() -> Fixture {
        let temp = tempfile::TempDir::new().unwrap();
        let (tx, events) = mpsc::unbounded_channel();
        let bars = Arc::new(InMemoryProgressBarStorage::default());
        let progress = Arc::new(ProgressServiceImpl::new(
            Arc::new(ChannelEmitter(tx)),
            bars.clone(),
        ));
        let client = Arc::new(FakeClient::default());
        Fixture {
            provider: Arc::new(AzulJreProvider::new(
                progress.clone(),
                client.clone(),
                client.clone(),
            )),
            client,
            progress,
            bars,
            install_dir: temp.path().join("java"),
            events,
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
            .install(VERSION, &fixture.install_dir, false)
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
            .install(VERSION, &fixture.install_dir, false)
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
            .install(VERSION, &fixture.install_dir, false)
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
            .install(VERSION, &fixture.install_dir, false)
            .await;

        assert!(
            matches!(result, Err(JavaDomainError::FailedToInstall { .. })),
            "{result:?}"
        );
        assert!(
            std::error::Error::source(&result.unwrap_err()).is_some(),
            "the cause of the failure is kept"
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

    /// A tree of `entries` (name, content) in a new directory.
    fn dir_with(parent: &Path, name: &str, entries: &[(&str, &str)]) -> PathBuf {
        let dir = parent.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        for (entry, content) in entries {
            std::fs::write(dir.join(entry), content).unwrap();
        }
        dir
    }

    fn rename_unless(
        refused: impl Fn(&Path, &Path) -> bool,
    ) -> impl Fn(&Path, &Path) -> std::io::Result<()> {
        move |from, to| {
            if refused(from, to) {
                Err(std::io::Error::other("refused"))
            } else {
                std::fs::rename(from, to)
            }
        }
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn should_put_the_previous_entries_back_when_a_later_entry_cannot_be_moved_in() {
        let temp = tempfile::TempDir::new().unwrap();
        let install = dir_with(temp.path(), "install", &[("a", "old a"), ("b", "old b")]);
        let staging = dir_with(temp.path(), "staging", &[("a", "new a"), ("b", "new b")]);
        let rename = rename_unless(|from, to| from.starts_with(&staging) && to.ends_with("b"));

        let result = swap_in(&staging, &install, VERSION, &rename);

        assert!(
            matches!(result, Err(JavaDomainError::FailedToInstall { .. })),
            "{result:?}"
        );
        assert_eq!(std::fs::read(install.join("a")).unwrap(), b"old a");
        assert_eq!(std::fs::read(install.join("b")).unwrap(), b"old b");
        assert_eq!(
            names(&install),
            ["a", "b"],
            "no set-aside directory is left"
        );
    }

    #[test]
    fn should_keep_the_set_aside_entries_when_they_cannot_be_put_back() {
        let temp = tempfile::TempDir::new().unwrap();
        let install = dir_with(temp.path(), "install", &[("b", "old b")]);
        let staging = dir_with(temp.path(), "staging", &[("b", "new b")]);
        // Neither the new entry going in nor the old one returning can take the place.
        let rename = rename_unless(|_, to| to == install.join("b"));

        let result = swap_in(&staging, &install, VERSION, &rename);

        assert!(result.is_err());
        let kept: Vec<_> = names(&install)
            .into_iter()
            .filter(|name| name.starts_with(ASIDE_PREFIX))
            .collect();
        assert_eq!(kept.len(), 1, "{:?}", names(&install));
        assert_eq!(
            std::fs::read(install.join(&kept[0]).join("b")).unwrap(),
            b"old b"
        );
    }

    #[test]
    fn should_restore_the_new_entry_when_the_previous_one_cannot_return() {
        let temp = tempfile::TempDir::new().unwrap();
        let install = dir_with(temp.path(), "install", &[("a", "old a"), ("b", "old b")]);
        let staging = dir_with(temp.path(), "staging", &[("a", "new a"), ("b", "new b")]);
        // The second entry cannot come in, and no previous entry can return.
        let moves_in = std::cell::Cell::new(0);
        let rename = |from: &Path, to: &Path| {
            if from.starts_with(&staging) {
                moves_in.set(moves_in.get() + 1);
                if moves_in.get() == 2 {
                    return Err(std::io::Error::other("refused"));
                }
            }
            if from.to_string_lossy().contains(ASIDE_PREFIX)
                && !from.to_string_lossy().contains(DISCARDED_DIR)
            {
                return Err(std::io::Error::other("refused"));
            }
            std::fs::rename(from, to)
        };

        let result = swap_in(&staging, &install, VERSION, &rename);

        assert!(result.is_err());
        let left: Vec<_> = names(&install)
            .into_iter()
            .filter(|name| !name.starts_with(".jre-"))
            .collect();
        assert_eq!(left.len(), 1, "the entry that came in stays: {left:?}");
        assert!(
            std::fs::read_to_string(install.join(&left[0]))
                .unwrap()
                .starts_with("new"),
        );
    }

    #[tokio::test]
    async fn should_finish_the_swap_when_the_install_is_dropped_between_its_two_moves() {
        let temp = tempfile::TempDir::new().unwrap();
        let install = dir_with(temp.path(), "install", &[("a", "old a")]);
        let staging = tempfile::Builder::new()
            .prefix(STAGING_PREFIX)
            .tempdir_in(&install)
            .unwrap();
        std::fs::write(staging.path().join("a"), "new a").unwrap();

        // The old entry is set aside; the swap waits there until the future is dropped.
        let (set_aside, set_aside_seen) = std::sync::mpsc::channel::<()>();
        let (release, released) = std::sync::mpsc::channel::<()>();
        let set_aside_once = Mutex::new(Some(set_aside));
        let released = Mutex::new(released);
        let swap = tokio::spawn(Provider::replace_installed_with(
            staging,
            install.clone(),
            VERSION,
            move |from, to| {
                std::fs::rename(from, to)?;
                if let Some(signal) = set_aside_once.lock().unwrap().take() {
                    signal.send(()).unwrap();
                    released.lock().unwrap().recv().unwrap();
                }
                Ok(())
            },
        ));
        tokio::task::spawn_blocking(move || set_aside_seen.recv().unwrap())
            .await
            .unwrap();
        swap.abort();
        let _ = swap.await;
        release.send(()).unwrap();

        // The blocking task is not stopped by the abort; it ends on its own.
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while names(&install).iter().any(|name| name.starts_with(".jre-"))
            && std::time::Instant::now() < deadline
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(std::fs::read_to_string(install.join("a")).unwrap(), "new a");
        assert_eq!(names(&install), ["a"]);
    }

    /// A directory with a file the game holds open cannot be moved on Windows.
    #[cfg(windows)]
    #[tokio::test]
    async fn should_keep_the_installed_jre_and_name_the_cause_when_the_old_one_cannot_be_moved() {
        use std::os::windows::fs::OpenOptionsExt;

        let fixture = fixture();
        std::fs::create_dir_all(installed_java(&fixture.install_dir).parent().unwrap()).unwrap();
        std::fs::write(installed_java(&fixture.install_dir), b"old jre").unwrap();
        let _held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(installed_java(&fixture.install_dir))
            .unwrap();
        serve_archive(&fixture.client, jre_zip(b"new jre")).await;

        let result = fixture
            .provider
            .install(VERSION, &fixture.install_dir, false)
            .await;

        let err = result.unwrap_err();
        assert!(
            matches!(err, JavaDomainError::RemoveOldInstallationError { .. }),
            "{err:?}"
        );
        assert!(std::error::Error::source(&err).is_some());
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
            .install(VERSION, &fixture.install_dir, false)
            .await;

        assert!(
            matches!(result, Err(JavaDomainError::VersionGetFailed { .. })),
            "{result:?}"
        );
        assert!(!installed_java(&fixture.install_dir).exists());
    }

    /// The bar of the Java download, once it is there.
    async fn java_bar(fixture: &Fixture) -> ProgressBar {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(bar) = fixture.bars.list().await.into_iter().next() {
                    return bar;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the bar never came")
    }

    struct InstallTask(tokio::task::JoinHandle<Result<PathBuf, JavaDomainError>>);

    impl InstallTask {
        async fn finished(self) -> Result<PathBuf, JavaDomainError> {
            tokio::time::timeout(Duration::from_secs(5), self.0)
                .await
                .expect("the install did not stop")
                .unwrap()
        }
    }

    /// Starts an install over an old JRE that waits for the archive, until `proceed`.
    async fn install_held(fixture: &Fixture, cancellable: bool) -> InstallTask {
        std::fs::create_dir_all(installed_java(&fixture.install_dir).parent().unwrap()).unwrap();
        std::fs::write(installed_java(&fixture.install_dir), b"old jre").unwrap();
        serve_archive(&fixture.client, jre_zip(b"new jre")).await;
        fixture.client.hold_archive.store(true, Ordering::SeqCst);
        let install = InstallTask(tokio::spawn({
            let provider = Arc::clone(&fixture.provider);
            let install_dir = fixture.install_dir.clone();
            async move { provider.install(VERSION, &install_dir, cancellable).await }
        }));
        fixture.client.at_archive.notified().await;
        install
    }

    #[tokio::test]
    async fn should_keep_the_installed_jre_and_end_the_bar_cancelled_when_called_off_while_downloading()
     {
        let mut fixture = fixture();
        let install = install_held(&fixture, true).await;
        let bar = java_bar(&fixture).await;
        assert_eq!(bar.cancel_state(), Some(CancelState::Open));

        fixture.progress.cancel_progress(bar.id).await.unwrap();
        let result = install.finished().await;

        assert!(
            matches!(result, Err(JavaDomainError::Cancelled)),
            "{result:?}"
        );
        assert_eq!(
            std::fs::read(installed_java(&fixture.install_dir)).unwrap(),
            b"old jre"
        );
        let last = sent(&mut fixture.events)
            .into_iter()
            .rfind(|event| event.progress_bar_id == bar.id)
            .unwrap();
        assert_eq!(last.status, ProgressStatus::Cancelled);
    }

    #[tokio::test]
    async fn should_refuse_to_replace_the_jre_when_called_off_before_the_point_of_no_return() {
        let fixture = fixture();
        let (bar, cancel) = fixture
            .progress
            .init_cancellable_shared_progress_safe(
                SharedDownload::Java { version: VERSION },
                100.0,
                String::new(),
            )
            .await;
        let bar = bar.unwrap();
        fixture.progress.cancel_progress(bar.id).await.unwrap();

        let passed = fixture
            .provider
            .pass_point_of_no_return(Some(&bar), &cancel)
            .await;

        assert!(
            matches!(passed, Err(JavaDomainError::Cancelled)),
            "{passed:?}"
        );
    }

    #[tokio::test]
    async fn should_keep_the_installed_jre_and_leave_no_staging_when_called_off_while_unpacking() {
        let fixture = fixture();
        std::fs::create_dir_all(installed_java(&fixture.install_dir).parent().unwrap()).unwrap();
        std::fs::write(installed_java(&fixture.install_dir), b"old jre").unwrap();
        let package = Package {
            download_url: ARCHIVE_URL.to_owned(),
            name: format!("{PACKAGE_DIR}.zip").into(),
            sha256_hash: String::new(),
        };
        let cancel = CancellationToken::new();
        cancel.cancel();

        let result = fixture
            .provider
            .unpack_jre(
                &fixture.install_dir,
                Bytes::from(jre_zip(b"new jre")),
                &package,
                VERSION,
                None,
                &cancel,
            )
            .await;

        assert!(
            matches!(result, Err(JavaDomainError::Cancelled)),
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
    async fn should_close_the_call_off_and_go_on_when_nobody_called_it_off() {
        let fixture = fixture();
        let (bar, cancel) = fixture
            .progress
            .init_cancellable_shared_progress_safe(
                SharedDownload::Java { version: VERSION },
                100.0,
                String::new(),
            )
            .await;
        let bar = bar.unwrap();

        fixture
            .provider
            .pass_point_of_no_return(Some(&bar), &cancel)
            .await
            .unwrap();
        fixture.progress.cancel_progress(bar.id).await.unwrap();

        assert!(!cancel.is_cancelled(), "a call-off after it is refused");
        let state = fixture.bars.get(bar.id).await.unwrap().cancel_state();
        assert_eq!(state, Some(CancelState::Closed));
    }

    #[tokio::test]
    async fn should_replace_the_jre_and_close_the_call_off_when_nobody_calls_it_off() {
        let mut fixture = fixture();
        let install = install_held(&fixture, true).await;
        let bar = java_bar(&fixture).await;
        fixture.client.proceed.notify_one();
        install.finished().await.unwrap();

        let states: Vec<_> = sent(&mut fixture.events)
            .into_iter()
            .filter(|event| event.progress_bar_id == bar.id)
            .filter_map(|event| event.cancel)
            .collect();
        assert_eq!(states.last(), Some(&CancelState::Closed), "{states:?}");
        assert_eq!(
            std::fs::read(installed_java(&fixture.install_dir)).unwrap(),
            b"new jre"
        );
    }

    #[tokio::test]
    async fn should_offer_no_call_off_when_the_install_is_a_part_of_another() {
        let fixture = fixture();
        let install = install_held(&fixture, false).await;

        let bar = java_bar(&fixture).await;
        assert_eq!(bar.cancel_state(), None);

        fixture.client.proceed.notify_one();
        install.finished().await.unwrap();
    }
}
