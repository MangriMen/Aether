use std::{
    fs,
    io::{Cursor, Read, copy},
    path::Path,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::plugins::{
        ExtractedPlugin, PluginContent, PluginError, PluginExtractor, PluginManifest,
    },
    shared::{io::domain::IoError, io::infra::read_async},
};

use super::ZipPluginExtractorConstants;

#[derive(Default)]
pub struct ZipPluginExtractor {
    constants: ZipPluginExtractorConstants,
}

impl ZipPluginExtractor {
    pub fn new(constants: ZipPluginExtractorConstants) -> Self {
        Self { constants }
    }

    fn read_manifest(
        &self,
        archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    ) -> Result<PluginManifest, PluginError> {
        let manifest_file = archive
            .by_name(self.constants.manifest_filename)
            .map_err(|_| PluginError::ManifestNotFound {
                path: self.constants.manifest_filename.to_string(),
            })?;

        let dto: aether_core_plugin_api::v0::PluginManifestDto =
            serde_json::from_reader(manifest_file).map_err(|e| {
                PluginError::InvalidManifestFormat {
                    error: e.to_string(),
                }
            })?;
        Ok(dto.try_into()?)
    }

    /// Extract every entry manually instead of using `ZipArchive::extract`, so that each path is
    /// validated with `enclosed_name` and the total unpacked size stays bounded.
    fn extract_entries(
        &self,
        archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
        temp_dir: &Path,
        source_path: &str,
    ) -> Result<(), PluginError> {
        let failed = || PluginError::FileExtractionFailed {
            from: source_path.to_string(),
        };

        if archive.len() > self.constants.max_entries {
            return Err(failed());
        }

        let mut total_bytes: u64 = 0;

        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|_| failed())?;

            // `enclosed_name` returns `None` for absolute paths and for paths escaping the root.
            let relative_path = entry.enclosed_name().ok_or_else(failed)?;
            let out_path = temp_dir.join(relative_path);

            // Symlinks could point outside `temp_dir`, so they are never materialized.
            if entry.is_symlink() {
                continue;
            }

            if entry.is_dir() {
                fs::create_dir_all(&out_path).map_err(|_| failed())?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|_| failed())?;
            }

            // Count actually decompressed bytes rather than the declared header size, which a
            // malicious archive can understate to slip past the limit.
            let remaining = self
                .constants
                .max_total_uncompressed_bytes
                .saturating_sub(total_bytes);
            let mut out_file = fs::File::create(&out_path).map_err(|_| failed())?;
            let written =
                copy(&mut (&mut entry).take(remaining + 1), &mut out_file).map_err(|_| failed())?;
            if written > remaining {
                return Err(failed());
            }
            total_bytes += written;
        }

        Ok(())
    }
}

#[async_trait]
impl PluginExtractor for ZipPluginExtractor {
    async fn extract(&self, file_path: &Path) -> Result<ExtractedPlugin, PluginError> {
        let source_path = file_path.to_string_lossy().to_string();

        let file = read_async(file_path)
            .await
            .map_err(|_| PluginError::ExtractionFailed {
                from: source_path.clone(),
            })?;

        let mut archive = zip::ZipArchive::new(Cursor::new(file))
            .map_err(|_| PluginError::InvalidExtractionFormat)?;

        let manifest = self.read_manifest(&mut archive)?;
        let plugin_id = manifest.metadata.id.clone();

        let temp_dir = TempDir::new().map_err(IoError::from)?;
        self.extract_entries(&mut archive, temp_dir.path(), &source_path)?;

        Ok(ExtractedPlugin {
            plugin_id,
            manifest,
            content: PluginContent::Filesystem { temp_dir },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Cursor, Write},
        path::PathBuf,
    };

    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::*;

    const MANIFEST: &str = r#"{
        "metadata": {
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "0.1.0",
            "description": null,
            "authors": [],
            "license": null
        },
        "runtime": { "allowedHosts": [], "allowedPaths": [] },
        "load": { "type": "extism", "file": "plugin.wasm", "memoryLimit": null },
        "api": { "version": "*", "features": [] }
    }"#;

    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        for (name, data) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    async fn write_zip(dir: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        let path = dir.join("plugin.zip");
        tokio::fs::write(&path, build_zip(entries)).await.unwrap();
        path
    }

    #[tokio::test]
    async fn should_reject_entry_escaping_extraction_root() {
        let source_dir = TempDir::new().unwrap();
        let escaped = std::env::temp_dir().join("aether-zip-slip-evil.txt");
        let _ = fs::remove_file(&escaped);

        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("../aether-zip-slip-evil.txt", b"pwned"),
            ],
        )
        .await;

        let result = ZipPluginExtractor::default().extract(&zip_path).await;

        assert!(matches!(
            result,
            Err(PluginError::FileExtractionFailed { .. })
        ));
        assert!(!escaped.exists());
    }

    #[tokio::test]
    async fn should_not_write_outside_root_for_absolute_entry_path() {
        let source_dir = TempDir::new().unwrap();
        let outside_dir = TempDir::new().unwrap();
        let outside_file = outside_dir.path().join("evil.txt");
        let absolute_entry = outside_file.to_string_lossy().to_string();

        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                (absolute_entry.as_str(), b"pwned"),
            ],
        )
        .await;

        // Absolute paths are stripped to a relative path by `enclosed_name`, so extraction may
        // succeed — what matters is that nothing lands outside the extraction root.
        let _ = ZipPluginExtractor::default().extract(&zip_path).await;

        assert!(!outside_file.exists());
    }

    #[tokio::test]
    async fn should_extract_regular_plugin_archive() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("plugin.wasm", b"\0asm"),
                ("assets/data.txt", b"hello"),
            ],
        )
        .await;

        let extracted = ZipPluginExtractor::default()
            .extract(&zip_path)
            .await
            .unwrap();

        assert_eq!(extracted.plugin_id, "test-plugin");
        let PluginContent::Filesystem { temp_dir } = &extracted.content;
        assert!(temp_dir.path().join("manifest.json").is_file());
        assert!(temp_dir.path().join("plugin.wasm").is_file());
        assert!(temp_dir.path().join("assets/data.txt").is_file());
    }

    #[tokio::test]
    async fn should_reject_archive_with_too_many_entries() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("plugin.wasm", b"\0asm"),
            ],
        )
        .await;

        let extractor = ZipPluginExtractor::new(ZipPluginExtractorConstants {
            max_entries: 1,
            ..Default::default()
        });

        assert!(matches!(
            extractor.extract(&zip_path).await,
            Err(PluginError::FileExtractionFailed { .. })
        ));
    }

    #[tokio::test]
    async fn should_reject_archive_exceeding_uncompressed_size_limit() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("plugin.wasm", &[0u8; 1024]),
            ],
        )
        .await;

        let extractor = ZipPluginExtractor::new(ZipPluginExtractorConstants {
            max_total_uncompressed_bytes: 16,
            ..Default::default()
        });

        assert!(matches!(
            extractor.extract(&zip_path).await,
            Err(PluginError::FileExtractionFailed { .. })
        ));
    }

    #[tokio::test]
    async fn should_reject_archive_whose_entries_exceed_limit_in_aggregate() {
        let source_dir = TempDir::new().unwrap();
        // Each entry is small enough on its own; only the running total crosses the limit.
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("a.bin", &[0u8; 64]),
                ("b.bin", &[0u8; 64]),
            ],
        )
        .await;

        let extractor = ZipPluginExtractor::new(ZipPluginExtractorConstants {
            max_total_uncompressed_bytes: 100,
            ..Default::default()
        });

        assert!(matches!(
            extractor.extract(&zip_path).await,
            Err(PluginError::FileExtractionFailed { .. })
        ));
    }
}
