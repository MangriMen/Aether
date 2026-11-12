use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

use crate::features::plugins::domain::PluginError;
use crate::shared::hash::HashAlgorithm;
use crate::shared::hash::infra::hash_file_hex_async;
use crate::shared::io::domain::IoError;

use super::PluginProvider;

/// A downloaded plugin archive; the temp file is deleted on drop.
pub struct DownloadedArchive {
    file: tempfile::NamedTempFile,
    sha256: String,
}

impl DownloadedArchive {
    pub fn path(&self) -> &Path {
        self.file.path()
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Downloads and hashes an archive; a failed or cut-short download leaves no file behind.
pub async fn download_to_temp_file(
    provider: &dyn PluginProvider,
    download_url: &str,
) -> Result<DownloadedArchive, PluginError> {
    let dir = plugin_archives_dir();
    crate::shared::io::infra::create_dir_all(&dir)
        .await
        .map_err(PluginError::Storage)?;
    download_to_temp_file_in(&dir, provider, download_url).await
}

/// Where downloaded plugin archives are put, apart from the rest of the temporary files so that
/// what a crash leaves behind can be cleared as a whole.
fn plugin_archives_dir() -> PathBuf {
    std::env::temp_dir().join("aether-plugin-archives")
}

/// Removes the archives that an earlier run left in the temporary directory. A failure is logged
/// and nothing more: they are only temporary files.
pub fn clean_plugin_archives() {
    remove_archives_dir(&plugin_archives_dir());
}

fn remove_archives_dir(dir: &Path) {
    match std::fs::remove_dir_all(dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => tracing::warn!(
            "Could not remove the plugin archives left in '{}': {e}",
            dir.display()
        ),
    }
}

async fn download_to_temp_file_in(
    dir: &Path,
    provider: &dyn PluginProvider,
    download_url: &str,
) -> Result<DownloadedArchive, PluginError> {
    let io = |e| PluginError::Storage(IoError::IoError(e));

    let temp_file = tempfile::NamedTempFile::new_in(dir).map_err(io)?;
    let mut file = tokio::fs::File::from_std(temp_file.reopen().map_err(io)?);

    provider.download_plugin(download_url, &mut file).await?;
    file.flush().await.map_err(io)?;

    // The digest is of the file on disk, which is what gets read afterwards, whatever the
    // provider wrote into it.
    let sha256 = hash_file_hex_async(HashAlgorithm::Sha256, temp_file.path())
        .await
        .map_err(io)?;

    Ok(DownloadedArchive {
        file: temp_file,
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::features::plugins::domain::{
        PluginProviderError, PluginSourceType, ProviderPluginPreview, ProviderUpdateInfo,
    };

    use super::*;

    /// Writes `body`, then fails if asked to, the way a connection that drops half-way does.
    struct PartialProvider {
        body: &'static [u8],
        fails: bool,
    }

    #[async_trait]
    impl PluginProvider for PartialProvider {
        async fn fetch_preview(
            &self,
            _: &str,
        ) -> Result<ProviderPluginPreview, PluginProviderError> {
            unimplemented!()
        }

        async fn fetch_latest_version(
            &self,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<ProviderUpdateInfo, PluginProviderError> {
            unimplemented!()
        }

        async fn download_plugin(
            &self,
            url: &str,
            dest: &mut tokio::fs::File,
        ) -> Result<u64, PluginProviderError> {
            dest.write_all(self.body).await.unwrap();
            if self.fails {
                return Err(PluginProviderError::DownloadFailed {
                    url: url.to_string(),
                    details: "connection reset".to_string(),
                });
            }
            Ok(self.body.len() as u64)
        }

        fn source_type(&self) -> PluginSourceType {
            PluginSourceType::GitHub
        }

        fn display_name(&self) -> String {
            "GitHub".to_owned()
        }

        fn parse_identifier(&self, raw: &str) -> Result<String, PluginProviderError> {
            Ok(raw.to_owned())
        }
    }

    #[test]
    fn should_remove_the_archives_left_behind_and_not_fail_when_there_is_no_directory() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("aether-plugin-archives");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("left.tmp"), b"archive").unwrap();

        remove_archives_dir(&dir);
        assert!(!dir.exists());

        remove_archives_dir(&dir);
    }

    #[tokio::test]
    async fn should_hand_over_the_whole_archive_in_a_file_when_the_download_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let provider = PartialProvider {
            body: b"archive",
            fails: false,
        };

        let file = download_to_temp_file_in(dir.path(), &provider, "u")
            .await
            .unwrap();

        assert_eq!(std::fs::read(file.path()).unwrap(), b"archive");
        assert_eq!(
            file.sha256(),
            crate::shared::hash::infra::sha256_hex(b"archive")
        );
    }

    #[tokio::test]
    async fn should_leave_no_file_behind_when_the_download_fails_half_way() {
        let dir = tempfile::tempdir().unwrap();
        let provider = PartialProvider {
            body: b"arch",
            fails: true,
        };

        let result = download_to_temp_file_in(dir.path(), &provider, "u").await;

        assert!(result.is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
