use std::{
    fs,
    io::{Read, Seek, copy},
    path::Path,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::plugins::{
        ArchiveMetadata, ExtractedPlugin, PluginCapabilities, PluginContent, PluginExtractor,
        PluginManifest, PluginProviderError, infra::PluginCapabilitiesV1,
    },
    shared::io::domain::IoError,
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

    /// One of the archive's small JSON files, read whole; `None` when the archive has no such
    /// entry. A file over the limit is refused instead of being taken into memory.
    fn read_small_entry<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
        name: &str,
    ) -> Result<Option<Vec<u8>>, PluginProviderError> {
        let entry = match archive.by_name(name) {
            Ok(entry) => entry,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(_) => return Err(PluginProviderError::InvalidExtractionFormat),
        };

        let limit = self.constants.max_metadata_bytes;
        let mut bytes = Vec::new();
        entry
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| PluginProviderError::InvalidExtractionFormat)?;
        if bytes.len() as u64 > limit {
            return Err(PluginProviderError::InvalidManifestFormat {
                error: format!("{name} is larger than {limit} bytes"),
            });
        }
        Ok(Some(bytes))
    }

    /// Refuses an archive that names one path twice, also when only the letter case differs:
    /// on a case-insensitive file system the later entry would replace the earlier one, so the
    /// files unpacked would not be the ones the manifest was read from.
    fn check_unique_names<R: Read + Seek>(
        archive: &zip::ZipArchive<R>,
    ) -> Result<(), PluginProviderError> {
        let mut seen = std::collections::HashSet::new();
        for name in archive.file_names() {
            let name = name.map_err(|_| PluginProviderError::InvalidExtractionFormat)?;
            let key = name.replace('\\', "/").trim_end_matches('/').to_lowercase();
            if !seen.insert(key) {
                return Err(PluginProviderError::InvalidExtractionFormat);
            }
        }
        Ok(())
    }

    fn read_manifest<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
    ) -> Result<PluginManifest, PluginProviderError> {
        let name = self.constants.manifest_filename;
        let bytes = self.read_small_entry(archive, name)?.ok_or_else(|| {
            PluginProviderError::ManifestNotFound {
                path: name.to_string(),
            }
        })?;

        let dto: aether_core_plugin_api::v0::PluginManifestDto = serde_json::from_slice(&bytes)
            .map_err(|e| PluginProviderError::InvalidManifestFormat {
                error: e.to_string(),
            })?;
        Ok(dto.try_into()?)
    }

    /// Reads `capabilities.json`, when the plugin has one, and checks it the way loading it
    /// would: a plugin whose capabilities would be refused on load is refused before any file is
    /// copied. Returns it as written.
    fn read_capabilities<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
    ) -> Result<Option<String>, PluginProviderError> {
        let invalid = |error: String| PluginProviderError::InvalidManifestFormat { error };
        let Some(bytes) = self.read_small_entry(archive, self.constants.capabilities_filename)?
        else {
            return Ok(None);
        };

        let dto: PluginCapabilitiesV1 =
            serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
        PluginCapabilities::try_from(dto)?.validate()?;

        String::from_utf8(bytes)
            .map(Some)
            .map_err(|e| invalid(e.to_string()))
    }

    /// The manifest and the capabilities of the archive, with the checks installing it makes.
    fn read_metadata_from<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
    ) -> Result<ArchiveMetadata, PluginProviderError> {
        Self::check_unique_names(archive)?;
        let manifest = self.read_manifest(archive)?;
        let capabilities = self.read_capabilities(archive)?;
        Ok(ArchiveMetadata {
            manifest,
            capabilities,
        })
    }

    fn open_archive(
        file: fs::File,
        source_path: &str,
    ) -> Result<zip::ZipArchive<std::io::BufReader<fs::File>>, PluginProviderError> {
        zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|err| match err {
            // A file that could not be read is not a file in the wrong format.
            zip::result::ZipError::Io(_) => PluginProviderError::ExtractionFailed {
                from: source_path.to_string(),
            },
            _ => PluginProviderError::InvalidExtractionFormat,
        })
    }

    /// Extract every entry manually instead of using `ZipArchive::extract`, so that each path is
    /// validated with `enclosed_name` and the total unpacked size stays bounded.
    fn extract_entries<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
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
        self.run_blocking(file_path, |extractor, file, source_path| {
            extractor.extract_blocking(file, source_path)
        })
        .await
    }

    async fn read_metadata(
        &self,
        file_path: &Path,
    ) -> Result<ArchiveMetadata, PluginProviderError> {
        self.run_blocking(file_path, |extractor, file, source_path| {
            let mut archive = Self::open_archive(file, source_path)?;
            extractor.read_metadata_from(&mut archive)
        })
        .await
    }
}

impl ZipPluginExtractor {
    /// Opens the archive and reads it with `work`. Unpacking is plain blocking I/O and
    /// decompression, up to the archive's limits: off the async workers. The archive is read
    /// from the file as it goes, never held in memory whole.
    async fn run_blocking<T, F>(&self, file_path: &Path, work: F) -> Result<T, PluginProviderError>
    where
        T: Send + 'static,
        F: FnOnce(&Self, fs::File, &str) -> Result<T, PluginProviderError> + Send + 'static,
    {
        let source_path = file_path.to_string_lossy().to_string();
        let extractor = Self {
            constants: self.constants.clone(),
        };
        let from = source_path.clone();
        let path = file_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file =
                fs::File::open(&path).map_err(|_| PluginProviderError::ExtractionFailed {
                    from: source_path.clone(),
                })?;
            work(&extractor, file, &source_path)
        })
        .await
        .map_err(|_| PluginProviderError::ExtractionFailed { from })?
    }

    fn extract_blocking(
        &self,
        file: fs::File,
        source_path: &str,
    ) -> Result<ExtractedPlugin, PluginProviderError> {
        let mut archive = Self::open_archive(file, source_path)?;

        let ArchiveMetadata { manifest, .. } = self.read_metadata_from(&mut archive)?;
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
    use rstest::rstest;
    use std::{
        io::{Cursor, Write},
        path::PathBuf,
    };

    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::*;
    use crate::features::instance::PluginId;
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
        eprintln!("RESULT {:?}", result.as_ref().map(|_| ()));

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

        // The entry is stripped to a relative path, so the archive is still extracted; what
        // matters is that nothing lands outside the extraction root.
        ZipPluginExtractor::default()
            .extract(&zip_path)
            .await
            .expect("an absolute entry path is made relative, not an error");

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

        assert_eq!(extracted.plugin_id, PluginId::from_static("test-plugin"));
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

    #[rstest]
    #[case::too_many_entries(
        &[("manifest.json", MANIFEST.as_bytes()), ("plugin.wasm", b"\0asm")],
        ZipPluginExtractorConstants { max_entries: 1, ..Default::default() }
    )]
    #[case::exceeds_the_uncompressed_size_limit(
        &[("manifest.json", MANIFEST.as_bytes()), ("plugin.wasm", &[0u8; 1024])],
        ZipPluginExtractorConstants { max_total_uncompressed_bytes: 16, ..Default::default() }
    )]
    // Each entry is small enough on its own; only the running total crosses the limit.
    #[case::entries_exceed_the_limit_in_aggregate(
        &[("manifest.json", MANIFEST.as_bytes()), ("a.bin", &[0u8; 64]), ("b.bin", &[0u8; 64])],
        ZipPluginExtractorConstants { max_total_uncompressed_bytes: 100, ..Default::default() }
    )]
    #[tokio::test]
    async fn should_reject_an_archive_when_it_crosses_a_limit(
        #[case] entries: &[(&str, &[u8])],
        #[case] constants: ZipPluginExtractorConstants,
    ) {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(source_dir.path(), entries).await;

        let extractor = ZipPluginExtractor::new(constants);

        assert!(matches!(
            extractor.extract(&zip_path).await,
            Err(PluginProviderError::FileExtractionFailed { .. })
        ));
    }

    #[tokio::test]
    async fn should_refuse_an_archive_naming_a_path_twice_or_in_another_letter_case() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("MANIFEST.JSON", MANIFEST.as_bytes()),
            ],
        )
        .await;

        let result = ZipPluginExtractor::default().read_metadata(&zip_path).await;

        assert!(matches!(
            result,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
    }

    #[tokio::test]
    async fn should_read_the_manifest_and_capabilities_without_unpacking_the_archive() {
        let source_dir = TempDir::new().unwrap();
        let capabilities = br#"{"importers": [], "updaters": []}"#;
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("capabilities.json", capabilities),
                // Not a plugin file the reading would touch: it would fail on an unpack.
                ("../escape.txt", b"x"),
            ],
        )
        .await;

        let metadata = ZipPluginExtractor::default()
            .read_metadata(&zip_path)
            .await
            .unwrap();

        assert_eq!(metadata.manifest.metadata.id.as_str(), "test-plugin");
        assert_eq!(
            metadata.capabilities.as_deref().map(str::as_bytes),
            Some(&capabilities[..])
        );
    }

    #[tokio::test]
    async fn should_read_no_capabilities_when_the_archive_has_none() {
        let source_dir = TempDir::new().unwrap();
        let zip_path =
            write_zip(source_dir.path(), &[("manifest.json", MANIFEST.as_bytes())]).await;

        let metadata = ZipPluginExtractor::default()
            .read_metadata(&zip_path)
            .await
            .unwrap();

        assert!(metadata.capabilities.is_none());
    }

    #[tokio::test]
    async fn should_refuse_to_read_metadata_when_the_manifest_is_invalid() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(source_dir.path(), &[("manifest.json", b"{ not json")]).await;

        let result = ZipPluginExtractor::default().read_metadata(&zip_path).await;

        assert!(matches!(
            result,
            Err(PluginProviderError::InvalidManifestFormat { .. })
        ));
    }

    #[tokio::test]
    async fn should_refuse_to_read_metadata_when_the_manifest_is_missing() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(source_dir.path(), &[("plugin.wasm", b"\0asm")]).await;

        let result = ZipPluginExtractor::default().read_metadata(&zip_path).await;

        assert!(matches!(
            result,
            Err(PluginProviderError::ManifestNotFound { .. })
        ));
    }
}
