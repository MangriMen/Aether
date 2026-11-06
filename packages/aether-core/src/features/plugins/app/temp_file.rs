use std::path::Path;

use tokio::io::AsyncWriteExt;

use crate::features::plugins::domain::PluginError;
use crate::shared::io::domain::IoError;

use super::PluginProvider;

/// Downloads a plugin archive into a temporary file and returns it; the reader opens it by path.
/// The file is deleted when the returned `NamedTempFile` is dropped, so a download that fails or
/// is cut short leaves nothing behind.
pub async fn download_to_temp_file(
    provider: &dyn PluginProvider,
    download_url: &str,
) -> Result<tempfile::NamedTempFile, PluginError> {
    download_to_temp_file_in(&std::env::temp_dir(), provider, download_url).await
}

async fn download_to_temp_file_in(
    dir: &Path,
    provider: &dyn PluginProvider,
    download_url: &str,
) -> Result<tempfile::NamedTempFile, PluginError> {
    let io = |e| PluginError::Storage(IoError::IoError(e));

    let temp_file = tempfile::NamedTempFile::new_in(dir).map_err(io)?;
    let mut file = tokio::fs::File::from_std(temp_file.reopen().map_err(io)?);

    provider.download_plugin(download_url, &mut file).await?;
    file.flush().await.map_err(io)?;

    Ok(temp_file)
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

        fn parse_identifier(&self, raw: &str) -> Result<String, PluginProviderError> {
            Ok(raw.to_owned())
        }
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
