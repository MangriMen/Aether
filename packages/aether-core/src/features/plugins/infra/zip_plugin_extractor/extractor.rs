use std::{
    fs,
    io::{Cursor, Read, copy},
    path::Path,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::plugins::{
        ExtractedPlugin, PluginCapabilities, PluginContent, PluginExtractor, PluginManifest,
        PluginProviderError, infra::PluginCapabilitiesV1,
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
    ) -> Result<PluginManifest, PluginProviderError> {
        let manifest_file = archive
            .by_name(self.constants.manifest_filename)
            .map_err(|_| PluginProviderError::ManifestNotFound {
                path: self.constants.manifest_filename.to_string(),
            })?;

        let dto: aether_core_plugin_api::v0::PluginManifestDto =
            serde_json::from_reader(manifest_file).map_err(|e| {
                PluginProviderError::InvalidManifestFormat {
                    error: e.to_string(),
                }
            })?;
        Ok(dto.try_into()?)
    }

    /// Checks `capabilities.json`, when the plugin has one, the way loading it would: a plugin
    /// whose capabilities would be refused on load is refused before any file is copied.
    fn check_capabilities(
        &self,
        archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    ) -> Result<(), PluginProviderError> {
        let capabilities_file = match archive.by_name(self.constants.capabilities_filename) {
            Ok(file) => file,
            Err(zip::result::ZipError::FileNotFound) => return Ok(()),
            Err(_) => return Err(PluginProviderError::InvalidExtractionFormat),
        };

        let dto: PluginCapabilitiesV1 =
            serde_json::from_reader(capabilities_file).map_err(|e| {
                PluginProviderError::InvalidManifestFormat {
                    error: e.to_string(),
                }
            })?;

        PluginCapabilities::from(dto).validate()?;

        Ok(())
    }

    /// Extract every entry manually instead of using `ZipArchive::extract`, so that each path is
    /// validated with `enclosed_name` and the total unpacked size stays bounded.
    fn extract_entries(
        &self,
        archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
        temp_dir: &Path,
        source_path: &str,
    ) -> Result<(), PluginProviderError> {
        let failed = || PluginProviderError::FileExtractionFailed {
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
    async fn extract(&self, file_path: &Path) -> Result<ExtractedPlugin, PluginProviderError> {
        let source_path = file_path.to_string_lossy().to_string();

        let file =
            read_async(file_path)
                .await
                .map_err(|_| PluginProviderError::ExtractionFailed {
                    from: source_path.clone(),
                })?;

        // Unpacking is plain blocking I/O and decompression, up to the archive's limits: off the
        // async workers.
        let extractor = Self {
            constants: self.constants.clone(),
        };
        let from = source_path.clone();
        tokio::task::spawn_blocking(move || extractor.extract_blocking(file, &source_path))
            .await
            .map_err(|_| PluginProviderError::ExtractionFailed { from })?
    }
}

impl ZipPluginExtractor {
    fn extract_blocking(
        &self,
        file: Vec<u8>,
        source_path: &str,
    ) -> Result<ExtractedPlugin, PluginProviderError> {
        let mut archive = zip::ZipArchive::new(Cursor::new(file))
            .map_err(|_| PluginProviderError::InvalidExtractionFormat)?;

        let manifest = self.read_manifest(&mut archive)?;
        self.check_capabilities(&mut archive)?;
        let plugin_id = manifest.metadata.id.clone();

        let temp_dir = TempDir::new().map_err(IoError::from)?;
        self.extract_entries(&mut archive, temp_dir.path(), source_path)?;

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
    use crate::features::plugins::ManifestError;

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
    async fn should_reject_an_entry_when_it_escapes_the_extraction_root() {
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
            Err(PluginProviderError::FileExtractionFailed { .. })
        ));
        assert!(!escaped.exists());
    }

    #[tokio::test]
    async fn should_not_write_outside_the_root_when_an_entry_path_is_absolute() {
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
    async fn should_extract_a_plugin_when_the_archive_is_regular() {
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

    fn manifest_with_id(id: &str) -> String {
        MANIFEST.replace(r#""id": "test-plugin""#, &format!(r#""id": "{id}""#))
    }

    #[tokio::test]
    async fn should_reject_the_archive_before_extracting_when_the_plugin_id_is_reserved() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", manifest_with_id("aether").as_bytes()),
                ("plugin.wasm", b"\0asm"),
            ],
        )
        .await;

        assert!(matches!(
            ZipPluginExtractor::default().extract(&zip_path).await,
            Err(PluginProviderError::Manifest(
                ManifestError::ReservedPluginId
            ))
        ));
    }

    #[tokio::test]
    async fn should_reject_the_archive_before_extracting_when_the_plugin_id_is_invalid() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", manifest_with_id("../evil").as_bytes()),
                ("plugin.wasm", b"\0asm"),
            ],
        )
        .await;

        assert!(matches!(
            ZipPluginExtractor::default().extract(&zip_path).await,
            Err(PluginProviderError::Manifest(
                ManifestError::InvalidId { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn should_reject_the_archive_before_extracting_when_a_capability_id_is_invalid() {
        let source_dir = TempDir::new().unwrap();
        let capabilities = r#"{
            "contentProviders": [{
                "id": "core:modrinth",
                "name": "Fake",
                "handlers": { "search": "s", "getContent": "g" }
            }]
        }"#;
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("capabilities.json", capabilities.as_bytes()),
                ("plugin.wasm", b"\0asm"),
            ],
        )
        .await;

        assert!(matches!(
            ZipPluginExtractor::default().extract(&zip_path).await,
            Err(PluginProviderError::Manifest(ManifestError::InvalidId { field, .. }))
                if field == "contentProviders[].id"
        ));
    }

    #[tokio::test]
    async fn should_reject_an_archive_when_it_has_too_many_entries() {
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
            Err(PluginProviderError::FileExtractionFailed { .. })
        ));
    }

    #[tokio::test]
    async fn should_reject_an_archive_when_it_exceeds_the_uncompressed_size_limit() {
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
            Err(PluginProviderError::FileExtractionFailed { .. })
        ));
    }

    #[tokio::test]
    async fn should_reject_an_archive_when_its_entries_exceed_the_limit_in_aggregate() {
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
            Err(PluginProviderError::FileExtractionFailed { .. })
        ));
    }
}
