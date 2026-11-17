use std::{
    collections::HashMap,
    fs,
    io::{Read, Seek, copy},
    path::Path,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::plugins::{
        ArchiveMetadata, ExtractedPlugin, ManifestFormatError, PluginCapabilities, PluginContent,
        PluginExtractor, PluginManifest, PluginProviderError, infra::PluginCapabilitiesV1,
    },
    shared::io::domain::IoError,
};

use super::ZipPluginExtractorConstants;

/// The entries of an archive by the path each is unpacked to, lowercased.
struct EntryIndex(HashMap<String, usize>);

impl EntryIndex {
    fn key(path: &Path) -> String {
        path.components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
            .to_lowercase()
    }

    /// The entry unpacked to `name`, a path relative to the plugin root written with `/`.
    fn get(&self, name: &str) -> Option<usize> {
        self.0.get(&name.to_lowercase()).copied()
    }
}

#[derive(Default)]
pub struct ZipPluginExtractor {
    constants: ZipPluginExtractorConstants,
}

impl ZipPluginExtractor {
    pub fn new(constants: ZipPluginExtractorConstants) -> Self {
        Self { constants }
    }

    /// Reads a small JSON entry whole, refusing one over `max_metadata_bytes`; `None` if absent.
    /// The entry is found by the path it is unpacked to (see [`EntryIndex`]), so what is read
    /// is the file that ends up on disk under `name`.
    fn read_small_entry<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
        entries: &EntryIndex,
        name: &str,
    ) -> Result<Option<Vec<u8>>, PluginProviderError> {
        let Some(index) = entries.get(name) else {
            return Ok(None);
        };
        let entry = archive
            .by_index(index)
            .map_err(|_| PluginProviderError::InvalidExtractionFormat)?;
        // Neither is unpacked as a file holding what would be read here.
        if entry.is_dir() || entry.is_symlink() {
            return Err(PluginProviderError::InvalidExtractionFormat);
        }

        let limit = self.constants.max_metadata_bytes;
        let mut bytes = Vec::new();
        entry
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| PluginProviderError::InvalidExtractionFormat)?;
        if bytes.len() as u64 > limit {
            return Err(PluginProviderError::InvalidManifestFormat {
                error: ManifestFormatError::TooLarge {
                    name: name.to_owned(),
                    limit,
                },
            });
        }
        Ok(Some(bytes))
    }

    /// Indexes the entries by the path each is unpacked to, refusing an archive where two land on
    /// one path. The path is the one `extract_entries` writes to (`enclosed_name`, which resolves
    /// `./`, `x/../` and a leading `/`), compared without regard to letter case: on a
    /// case-insensitive file system the later entry replaces the earlier one. Either way the
    /// files unpacked would not be the ones the manifest was read from. An entry that has no
    /// such path is refused too: it could not be unpacked.
    fn index_entries<R: Read + Seek>(
        archive: &mut zip::ZipArchive<R>,
    ) -> Result<EntryIndex, PluginProviderError> {
        let invalid = || PluginProviderError::InvalidExtractionFormat;
        let mut paths = HashMap::new();
        for index in 0..archive.len() {
            let entry = archive.by_index_raw(index).map_err(|_| invalid())?;
            let path = entry.enclosed_name().ok_or_else(invalid)?;
            if paths.insert(EntryIndex::key(&path), index).is_some() {
                return Err(invalid());
            }
        }
        Ok(EntryIndex(paths))
    }

    fn read_manifest<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
        entries: &EntryIndex,
    ) -> Result<PluginManifest, PluginProviderError> {
        let name = self.constants.manifest_filename;
        let bytes = self
            .read_small_entry(archive, entries, name)?
            .ok_or_else(|| PluginProviderError::ManifestNotFound {
                path: name.to_string(),
            })?;

        let dto: aether_core_plugin_api::v0::PluginManifestDto = serde_json::from_slice(&bytes)
            .map_err(|e| PluginProviderError::InvalidManifestFormat { error: e.into() })?;
        Ok(dto.try_into()?)
    }

    /// Reads and validates `capabilities.json` before any file is copied; returns it as written.
    fn read_capabilities<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
        entries: &EntryIndex,
    ) -> Result<Option<String>, PluginProviderError> {
        let invalid =
            |error: ManifestFormatError| PluginProviderError::InvalidManifestFormat { error };
        let Some(bytes) =
            self.read_small_entry(archive, entries, self.constants.capabilities_filename)?
        else {
            return Ok(None);
        };

        let dto: PluginCapabilitiesV1 =
            serde_json::from_slice(&bytes).map_err(|e| invalid(e.into()))?;
        PluginCapabilities::try_from(dto)?.validate()?;

        String::from_utf8(bytes)
            .map(Some)
            .map_err(|e| invalid(e.into()))
    }

    fn read_metadata_from<R: Read + Seek>(
        &self,
        archive: &mut zip::ZipArchive<R>,
    ) -> Result<ArchiveMetadata, PluginProviderError> {
        let entries = Self::index_entries(archive)?;
        let manifest = self.read_manifest(archive, &entries)?;
        let capabilities = self.read_capabilities(archive, &entries)?;
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
    /// Blocking I/O and decompression run off the async workers; the archive is read from disk,
    /// never held whole.
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
        "api": { "version": "*" }
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

        // Refused while the archive is read, before anything is unpacked.
        assert!(matches!(
            result,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
        assert!(!escaped.exists());
    }

    #[tokio::test]
    async fn should_refuse_to_read_metadata_when_an_entry_escapes_the_root() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("../escape.txt", b"x"),
            ],
        )
        .await;

        let result = ZipPluginExtractor::default().read_metadata(&zip_path).await;

        assert!(matches!(
            result,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
    }

    /// The manifest of another plugin, the one an alias would put in place of the first.
    fn evil_manifest() -> String {
        MANIFEST
            .replace("test-plugin", "evil-plugin")
            .replace(r#""allowedHosts": []"#, r#""allowedHosts": ["*"]"#)
    }

    const CAPABILITIES: &str =
        r#"{"importers": [{"id": "benign", "name": "B", "handler": "import"}]}"#;
    const EVIL_CAPABILITIES: &str =
        r#"{"importers": [{"id": "evil", "name": "E", "handler": "import"}]}"#;

    /// An entry under a second name that unpacks over the first one: what would be read is the
    /// first entry, what would be installed the second. Such an archive is refused.
    #[rstest]
    #[case::current_dir("./{}")]
    #[case::parent_of_a_dir("x/../{}")]
    #[case::absolute("/{}")]
    #[case::backslash(".\\{}")]
    #[case::other_case("X/../{}")]
    #[tokio::test]
    async fn should_refuse_an_archive_when_an_entry_unpacks_over_the_manifest(#[case] alias: &str) {
        let source_dir = TempDir::new().unwrap();
        let evil = evil_manifest();
        let alias = alias.replace("{}", "manifest.json");
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("plugin.wasm", b"\0asm"),
                (alias.as_str(), evil.as_bytes()),
            ],
        )
        .await;

        let extractor = ZipPluginExtractor::default();

        assert!(matches!(
            extractor.read_metadata(&zip_path).await,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
        assert!(matches!(
            extractor.extract(&zip_path).await,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
    }

    #[rstest]
    #[case::current_dir("./{}")]
    #[case::parent_of_a_dir("x/../{}")]
    #[case::absolute("/{}")]
    #[case::backslash(".\\{}")]
    #[case::other_case("{}")]
    #[tokio::test]
    async fn should_refuse_an_archive_when_an_entry_unpacks_over_the_capabilities(
        #[case] alias: &str,
    ) {
        let source_dir = TempDir::new().unwrap();
        // The other-case alias is of the upper-case name, so the two differ in case only.
        let alias = if alias == "{}" {
            "CAPABILITIES.JSON".to_owned()
        } else {
            alias.replace("{}", "capabilities.json")
        };
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("capabilities.json", CAPABILITIES.as_bytes()),
                ("plugin.wasm", b"\0asm"),
                (alias.as_str(), EVIL_CAPABILITIES.as_bytes()),
            ],
        )
        .await;

        let extractor = ZipPluginExtractor::default();

        assert!(matches!(
            extractor.read_metadata(&zip_path).await,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
        assert!(matches!(
            extractor.extract(&zip_path).await,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
    }

    /// With no plain entry beside it, an entry unpacked to `manifest.json` is the manifest: what
    /// is read is what is unpacked.
    #[rstest]
    #[case::current_dir("./manifest.json")]
    #[case::absolute("/manifest.json")]
    #[case::other_case("Manifest.JSON")]
    #[tokio::test]
    async fn should_read_the_manifest_from_the_entry_unpacked_to_it(#[case] name: &str) {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[(name, MANIFEST.as_bytes()), ("plugin.wasm", b"\0asm")],
        )
        .await;

        let metadata = ZipPluginExtractor::default()
            .read_metadata(&zip_path)
            .await
            .unwrap();

        assert_eq!(metadata.manifest.metadata.id.as_str(), "test-plugin");
    }

    /// Unpacked as `CAPABILITIES.JSON`, it is what a case-insensitive file system gives the
    /// launcher as `capabilities.json`: it is read and checked like that name.
    #[tokio::test]
    async fn should_read_the_capabilities_when_their_entry_differs_in_case() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(
            source_dir.path(),
            &[
                ("manifest.json", MANIFEST.as_bytes()),
                ("CAPABILITIES.JSON", EVIL_CAPABILITIES.as_bytes()),
            ],
        )
        .await;

        let metadata = ZipPluginExtractor::default()
            .read_metadata(&zip_path)
            .await
            .unwrap();

        assert!(metadata.capabilities.unwrap().contains("evil"));
    }

    #[tokio::test]
    async fn should_refuse_a_manifest_when_its_entry_is_a_directory() {
        let source_dir = TempDir::new().unwrap();
        let zip_path = write_zip(source_dir.path(), &[("manifest.json/", b"")]).await;

        let result = ZipPluginExtractor::default().read_metadata(&zip_path).await;

        assert!(matches!(
            result,
            Err(PluginProviderError::InvalidExtractionFormat)
        ));
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
