//! A `.mrpack` read into plan data: its index becomes the plan's files, its `overrides/` (and
//! those of each side) the plan's overrides, its dependencies the game the pack runs on. Nothing
//! here touches the network: the archive is already on disk.

use std::{io::Read, path::Path, sync::Arc};

use crate::{
    features::instance::{
        AnchoredPath, ContentId, ContentIdentity, ContentPath, ContentProviderRejection,
        ContentSource, ContentType, ContentVersionId, FileSide, InstallFileEntry, InstanceError,
        OptionalFile, OverrideLocation, OverrideSource, PackMetadata, PlatformId, StorageError,
        infra::content_providers::modrinth::api_client::{
            Hashes, ModrinthEnv, ModrinthIndex, ModrinthIndexFile,
        },
    },
    features::minecraft::{GameVersion, LoaderVersionId, ModLoader},
    shared::{
        hash::{FileHash, HashAlgorithm},
        limits::{MAX_MRPACK_FILES, MAX_MRPACK_INDEX_BYTES},
        web_url::HttpsUrl,
    },
};

use super::ModrinthContentProvider;

pub(super) const INDEX_PATH: &str = "modrinth.index.json";
/// The overrides every side gets, then those of one side only, laid over them.
const OVERRIDES_DIRS: [(&str, FileSide); 3] = [
    ("overrides", FileSide::Both),
    ("client-overrides", FileSide::Client),
    ("server-overrides", FileSide::Server),
];

const MODRINTH_CDN_HOST: &str = "cdn.modrinth.com";

/// The hosts the `.mrpack` format allows downloads from. A pack is a file anyone can publish
/// or hand over, and its URLs are downloaded with the launcher's own unrestricted client, so a
/// URL elsewhere (a LAN address, any other site) is refused rather than fetched.
const ALLOWED_DOWNLOAD_HOSTS: [&str; 4] = [
    MODRINTH_CDN_HOST,
    "github.com",
    "raw.githubusercontent.com",
    "gitlab.com",
];

fn rejected(reason: ContentProviderRejection) -> InstanceError {
    InstanceError::ContentProviderError { reason }
}

/// Read `modrinth.index.json` out of the archive. Blocking.
///
/// The index is refused past [`MAX_MRPACK_INDEX_BYTES`] decompressed or [`MAX_MRPACK_FILES`]
/// files, so a crafted pack cannot exhaust memory or plan an unbounded number of downloads.
pub(super) fn read_index(mrpack_path: &Path) -> Result<ModrinthIndex, InstanceError> {
    read_index_within(mrpack_path, MAX_MRPACK_INDEX_BYTES, MAX_MRPACK_FILES)
}

fn read_index_within(
    mrpack_path: &Path,
    max_bytes: u64,
    max_files: usize,
) -> Result<ModrinthIndex, InstanceError> {
    let file = std::fs::File::open(mrpack_path)
        .map_err(|e| rejected(ContentProviderRejection::PackUnreadable(Arc::new(e))))?;

    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        rejected(ContentProviderRejection::PackInvalidArchive(Arc::new(
            e.into(),
        )))
    })?;

    let index_file = archive
        .by_name(INDEX_PATH)
        .map_err(|_| rejected(ContentProviderRejection::PackNoIndex { index: INDEX_PATH }))?;

    // One byte past the limit tells an index that is too large from one that fits exactly.
    let mut bytes = Vec::new();
    index_file
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| rejected(ContentProviderRejection::PackUnreadable(Arc::new(e))))?;
    if bytes.len() as u64 > max_bytes {
        return Err(rejected(ContentProviderRejection::PackIndexTooLarge {
            max_bytes,
        }));
    }

    let index: ModrinthIndex = serde_json::from_slice(&bytes)
        .map_err(|e| rejected(ContentProviderRejection::PackIndexMalformed(Arc::new(e))))?;

    if index.files.len() > max_files {
        return Err(rejected(ContentProviderRejection::TooManyFiles {
            count: index.files.len(),
            max: max_files,
        }));
    }

    if index.game != "minecraft" {
        return Err(rejected(ContentProviderRejection::PackWrongGame {
            game: index.game,
        }));
    }

    Ok(index)
}

/// [`read_index`] off the async runtime.
pub(super) async fn read_index_async(mrpack_path: &Path) -> Result<ModrinthIndex, InstanceError> {
    let mrpack_path = mrpack_path.to_owned();
    tokio::task::spawn_blocking(move || read_index(&mrpack_path))
        .await
        .map_err(|e| InstanceError::storage(StorageError::other(e)))?
}

/// The project and version a Modrinth CDN URL names:
/// `https://cdn.modrinth.com/data/<project>/versions/<version>/<file>`.
pub(super) fn parse_cdn_url(url: &HttpsUrl) -> Option<(ContentId, ContentVersionId)> {
    if url.host_str() != MODRINTH_CDN_HOST {
        return None;
    }

    match url.as_url().path_segments()?.collect::<Vec<_>>()[..] {
        ["data", project_id, "versions", version_id, _, ..]
            if !project_id.is_empty() && !version_id.is_empty() =>
        {
            Some((
                ContentId::try_from(project_id).ok()?,
                ContentVersionId::try_from(version_id).ok()?,
            ))
        }
        _ => None,
    }
}

/// Which Modrinth content a URL is, when it is a file on Modrinth's CDN.
pub(super) fn cdn_identity(url: &HttpsUrl) -> Option<ContentIdentity> {
    parse_cdn_url(url).map(|(content_id, version)| ContentIdentity {
        platform: PlatformId::from_static(ModrinthContentProvider::PLATFORM),
        content_id,
        version,
    })
}

/// The file's SHA-512, the stronger of the two hashes Modrinth gives, validated.
pub(super) fn sha512_of(hashes: &Hashes) -> Result<FileHash, InstanceError> {
    FileHash::new(HashAlgorithm::Sha512, hashes.sha512.clone())
        .map_err(|e| rejected(ContentProviderRejection::InvalidHash(e)))
}

/// Every file of the index as a plan entry — optional and server-only ones too: leaving files
/// out is the core's decision.
pub(super) fn plan_files(index: &ModrinthIndex) -> Result<Vec<InstallFileEntry>, InstanceError> {
    index.files.iter().map(plan_file).collect()
}

/// The override directories of the archive, every side's included: as with the files, leaving
/// out the server's is the core's decision. A directory the archive lacks lists no files.
/// The archive is the core's own download, so it is anchored at the directory it is in.
pub(super) fn plan_overrides(archive_path: &Path) -> Vec<OverrideSource> {
    OVERRIDES_DIRS
        .iter()
        .map(|&(subdir, side)| {
            OverrideSource::new(
                OverrideLocation::Archive {
                    archive_path: AnchoredPath::at_parent(archive_path),
                    subdir: subdir.to_owned(),
                },
                side,
            )
        })
        .collect()
}

fn plan_file(file: &ModrinthIndexFile) -> Result<InstallFileEntry, InstanceError> {
    let url = file
        .downloads
        .iter()
        .find_map(|url| allowed_download(url))
        .ok_or_else(|| {
            rejected(ContentProviderRejection::NoAllowedDownload {
                path: file.path.clone(),
            })
        })?;

    let (side, optional) = side_and_option(file.env.as_ref());
    let hash = sha512_of(&file.hashes)?;
    let relative_path = ContentPath::parse(&file.path).inspect_err(|err| {
        tracing::warn!("The pack lists a file at an invalid path: {err}");
    })?;
    let content_type = content_type_of(relative_path.as_str());

    Ok(InstallFileEntry {
        optional,
        side,
        size: Some(file.file_size),
        ..InstallFileEntry::new(
            ContentSource::Direct {
                identity: cdn_identity(&url),
                url,
                hash: Some(hash),
            },
            relative_path,
            content_type,
        )
    })
}

/// `url` as a download, if it is an `https` URL on a host the format allows.
fn allowed_download(url: &str) -> Option<HttpsUrl> {
    HttpsUrl::parse(url)
        .ok()
        .filter(|url| ALLOWED_DOWNLOAD_HOSTS.contains(&url.host_str()))
}

/// The content type by the top folder of the path. A pack may place a file anywhere; outside
/// the content folders the type is only a label (nothing in the executor depends on it), so
/// such a file is still installed, as a mod.
fn content_type_of(relative_path: &str) -> ContentType {
    let folder = relative_path.split(['/', '\\']).next().unwrap_or_default();

    ContentType::iterator()
        .filter(|content_type| *content_type != ContentType::Modpack)
        .find(|content_type| content_type.get_folder() == folder)
        .unwrap_or(ContentType::Mod)
}

/// The side a file is for and, for the client, whether it is optional. `env` values are
/// `required`, `optional` and `unsupported`; an absent `env` means required on both sides.
///
/// An optional file starts selected, as the Modrinth App installs it.
fn side_and_option(env: Option<&ModrinthEnv>) -> (FileSide, Option<OptionalFile>) {
    const UNSUPPORTED: &str = "unsupported";

    let Some(env) = env else {
        return (FileSide::Both, None);
    };

    // A file for neither side is not the client's either: as `Server`, a client skips it.
    let side = match (env.client.as_str(), env.server.as_str()) {
        (UNSUPPORTED, _) => FileSide::Server,
        (_, UNSUPPORTED) => FileSide::Client,
        _ => FileSide::Both,
    };
    let optional = (env.client == "optional").then_some(OptionalFile {
        default_enabled: true,
        description: None,
    });

    (side, optional)
}

fn resolve_loader_from_manifest(manifest: &ModrinthIndex) -> (ModLoader, Option<String>) {
    let loaders = &manifest.dependencies.loaders;

    if let Some(v) = loaders.get("fabric-loader") {
        return (ModLoader::Fabric, Some(v.clone()));
    }
    if let Some(v) = loaders.get("forge") {
        return (ModLoader::Forge, Some(v.clone()));
    }
    if let Some(v) = loaders.get("quilt-loader") {
        return (ModLoader::Quilt, Some(v.clone()));
    }
    if let Some(v) = loaders.get("neoforge") {
        return (ModLoader::NeoForge, Some(v.clone()));
    }

    (ModLoader::Vanilla, None)
}

/// The pack's description from its index. `version` — the Modrinth version the archive is the
/// file of, when it came from the platform; `icon_url` — the project's.
pub(super) fn pack_metadata(
    index: &ModrinthIndex,
    version_id: Option<ContentVersionId>,
    icon_url: Option<HttpsUrl>,
) -> Result<PackMetadata, InstanceError> {
    let (mod_loader, loader_version) = resolve_loader_from_manifest(index);

    Ok(PackMetadata {
        name: index.name.clone(),
        version_id,
        version_name: Some(index.version_id.clone()),
        description: index.summary.clone(),
        game_version: GameVersion::parse(&index.dependencies.minecraft)?,
        mod_loader,
        loader_version: loader_version.map(LoaderVersionId::try_from).transpose()?,
        file_count: Some(index.files.len()),
        icon_url,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::features::{
        instance::infra::content_providers::modrinth::api_client::ModrinthDependencies,
        minecraft::ModLoader,
    };

    use super::*;

    const SODIUM_URL: &str = "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d/sodium.jar";

    fn hashes() -> Hashes {
        Hashes {
            sha1: "a".repeat(40),
            sha512: "B".repeat(128),
        }
    }

    fn index_file(path: &str, downloads: &[&str], env: Option<(&str, &str)>) -> ModrinthIndexFile {
        ModrinthIndexFile {
            path: path.into(),
            hashes: hashes(),
            env: env.map(|(client, server)| ModrinthEnv {
                client: client.into(),
                server: server.into(),
            }),
            downloads: downloads.iter().map(|url| (*url).to_owned()).collect(),
            file_size: 42,
        }
    }

    fn index(files: Vec<ModrinthIndexFile>) -> ModrinthIndex {
        ModrinthIndex {
            format_version: 1,
            game: "minecraft".into(),
            version_id: "1.4.2".into(),
            name: "Pack".into(),
            summary: Some("A pack".into()),
            files,
            dependencies: ModrinthDependencies {
                minecraft: "1.21.1".into(),
                loaders: HashMap::from([("neoforge".into(), "21.1.77".into())]),
            },
        }
    }

    /// A `.mrpack` holding only `modrinth.index.json` with `body`, deflated.
    fn mrpack_with_index(dir: &tempfile::TempDir, body: &[u8]) -> std::path::PathBuf {
        use std::io::Write;

        let path = dir.path().join("pack.mrpack");
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        writer
            .start_file(INDEX_PATH, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(body).unwrap();
        writer.finish().unwrap();
        path
    }

    #[test]
    fn should_read_an_index_when_it_is_within_the_limits() {
        let dir = tempfile::TempDir::new().unwrap();
        let body = serde_json::to_vec(&index(vec![index_file("mods/a.jar", &[], None)])).unwrap();
        let path = mrpack_with_index(&dir, &body);

        let read = read_index_within(&path, body.len() as u64, 1).unwrap();

        assert_eq!(read.files.len(), 1);
    }

    /// Whitespace deflates to almost nothing: a tiny archive whose index inflates past the limit
    /// is refused without being read to its end.
    #[test]
    fn should_refuse_an_index_when_it_inflates_past_the_limit() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut body = serde_json::to_vec(&index(Vec::new())).unwrap();
        let padding = usize::try_from(MAX_MRPACK_INDEX_BYTES).unwrap();
        body.resize(body.len() + padding, b' ');
        let path = mrpack_with_index(&dir, &body);
        assert!(std::fs::metadata(&path).unwrap().len() < 1024 * 1024);

        let err = read_index(&path).unwrap_err();

        assert!(
            matches!(
                err,
                InstanceError::ContentProviderError {
                    reason: ContentProviderRejection::PackIndexTooLarge { .. }
                }
            ),
            "{err}"
        );
    }

    #[test]
    fn should_refuse_an_index_when_it_lists_too_many_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let files = (0..3)
            .map(|i| index_file(&format!("mods/{i}.jar"), &[], None))
            .collect();
        let path = mrpack_with_index(&dir, &serde_json::to_vec(&index(files)).unwrap());

        let err = read_index_within(&path, MAX_MRPACK_INDEX_BYTES, 2).unwrap_err();

        assert!(
            matches!(
                err,
                InstanceError::ContentProviderError {
                    reason: ContentProviderRejection::TooManyFiles { count: 3, max: 2 }
                }
            ),
            "{err}"
        );
    }

    #[test]
    fn should_read_the_project_and_version_when_given_a_cdn_url() {
        assert_eq!(
            parse_cdn_url(&HttpsUrl::parse(SODIUM_URL).unwrap()),
            Some((
                ContentId::try_from("AANobbMI").unwrap(),
                ContentVersionId::try_from("IZskON6d").unwrap()
            ))
        );
        for url in [
            "https://example.com/data/AANobbMI/versions/IZskON6d/sodium.jar",
            "https://cdn.modrinth.com/data/AANobbMI/sodium.jar",
            "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d",
        ] {
            assert_eq!(parse_cdn_url(&HttpsUrl::parse(url).unwrap()), None, "{url}");
        }
    }

    /// A path that leaves the instance or points into `.metadata` fails the plan where the index is
    /// read, before anything is resolved or downloaded — `.metadata` is the launcher's, the
    /// built-in provider's plan included.
    #[test]
    fn should_refuse_the_plan_when_a_file_path_is_not_a_content_path() {
        for path in [
            "../escape.jar",
            "/etc/passwd",
            "mods/../../escape.jar",
            "C:/x.jar",
            ".metadata/instance.json",
            ".METADATA/instance.json",
            ".metadata/plugins/packwiz/packwiz.toml",
        ] {
            let err = plan_files(&index(vec![
                index_file("mods/a.jar", &[SODIUM_URL], None),
                index_file(path, &[SODIUM_URL], None),
            ]))
            .unwrap_err();

            assert!(
                matches!(err, InstanceError::InvalidRelativePath { .. }),
                "{path}: {err:?}"
            );
        }
    }

    /// A file of Modrinth's CDN names its content, so the executor treats it
    /// as that mod across updates; a file elsewhere stays plain bytes.
    #[test]
    fn should_plan_index_files_as_direct_downloads_when_they_come_from_the_cdn() {
        let github = "https://github.com/owner/repo/releases/download/v1/tool.jar";
        let files = plan_files(&index(vec![
            index_file("mods/sodium.jar", &[SODIUM_URL], None),
            index_file("mods/tool.jar", &[github], None),
        ]))
        .unwrap();

        assert_eq!(
            files[0].source,
            ContentSource::Direct {
                url: HttpsUrl::parse(SODIUM_URL).unwrap(),
                hash: Some(FileHash::new(HashAlgorithm::Sha512, "b".repeat(128)).unwrap()),
                identity: Some(ContentIdentity {
                    platform: PlatformId::from_static("modrinth"),
                    content_id: ContentId::try_from("AANobbMI").unwrap(),
                    version: ContentVersionId::try_from("IZskON6d").unwrap(),
                }),
            }
        );
        assert_eq!(files[0].relative_path.as_str(), "mods/sodium.jar");
        assert_eq!(files[0].size, Some(42));
        assert!(matches!(
            &files[1].source,
            ContentSource::Direct { url, identity: None, .. } if url.as_str() == github
        ));
    }

    #[test]
    fn should_take_the_first_download_when_several_are_on_an_allowed_host() {
        let files = plan_files(&index(vec![index_file(
            "mods/sodium.jar",
            &[
                "http://cdn.modrinth.com/data/a/versions/b/c.jar",
                "https://192.168.0.1/a.jar",
                SODIUM_URL,
            ],
            None,
        )]))
        .unwrap();

        assert!(matches!(
            &files[0].source,
            ContentSource::Direct { url, .. } if url.as_str() == SODIUM_URL
        ));
    }

    #[test]
    fn should_refuse_a_file_when_no_download_is_on_an_allowed_host() {
        let result = plan_files(&index(vec![index_file(
            "mods/evil.jar",
            &["https://localhost/evil.jar", "https://example.com/evil.jar"],
            None,
        )]));

        assert!(matches!(
            result,
            Err(InstanceError::ContentProviderError {
                reason: ContentProviderRejection::NoAllowedDownload { .. }
            })
        ));
    }

    #[test]
    fn should_keep_the_parse_error_when_the_index_is_malformed() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = mrpack_with_index(&dir, b"{not json");

        let err = read_index(&path).unwrap_err();

        assert!(matches!(
            err,
            InstanceError::ContentProviderError {
                reason: ContentProviderRejection::PackIndexMalformed(_)
            }
        ));
    }

    #[test]
    fn should_refuse_a_pack_for_another_game() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut other = index(Vec::new());
        other.game = "hytale".into();
        let path = mrpack_with_index(&dir, &serde_json::to_vec(&other).unwrap());

        let err = read_index(&path).unwrap_err();

        assert!(matches!(
            err,
            InstanceError::ContentProviderError {
                reason: ContentProviderRejection::PackWrongGame { ref game }
            } if game == "hytale"
        ));
    }

    #[test]
    fn should_tell_a_missing_file_from_a_damaged_archive() {
        let dir = tempfile::TempDir::new().unwrap();
        let missing = read_index(&dir.path().join("none.mrpack")).unwrap_err();
        let damaged_path = dir.path().join("bad.mrpack");
        std::fs::write(&damaged_path, b"not a zip").unwrap();
        let damaged = read_index(&damaged_path).unwrap_err();

        assert!(matches!(
            missing,
            InstanceError::ContentProviderError {
                reason: ContentProviderRejection::PackUnreadable(_)
            }
        ));
        assert!(matches!(
            damaged,
            InstanceError::ContentProviderError {
                reason: ContentProviderRejection::PackInvalidArchive(_)
            }
        ));
    }

    #[test]
    fn should_map_env_to_side_and_optional_when_the_index_declares_it() {
        let files = plan_files(&index(vec![
            index_file(
                "mods/both.jar",
                &[SODIUM_URL],
                Some(("required", "required")),
            ),
            index_file(
                "mods/client.jar",
                &[SODIUM_URL],
                Some(("required", "unsupported")),
            ),
            index_file(
                "mods/server.jar",
                &[SODIUM_URL],
                Some(("unsupported", "required")),
            ),
            index_file(
                "mods/optional.jar",
                &[SODIUM_URL],
                Some(("optional", "optional")),
            ),
            index_file(
                "mods/neither.jar",
                &[SODIUM_URL],
                Some(("unsupported", "unsupported")),
            ),
        ]))
        .unwrap();

        let sides: Vec<_> = files.iter().map(|file| file.side).collect();
        assert_eq!(
            sides,
            [
                FileSide::Both,
                FileSide::Client,
                FileSide::Server,
                FileSide::Both,
                FileSide::Server,
            ]
        );
        assert!(files[..3].iter().all(|file| file.optional.is_none()));
        assert_eq!(
            files[3].optional,
            Some(OptionalFile {
                default_enabled: true,
                description: None
            })
        );
    }

    #[test]
    fn should_plan_overrides_of_every_side_when_they_are_layered() {
        let overrides = plan_overrides(Path::new("cache/pack.mrpack"));

        let planned: Vec<_> = overrides
            .iter()
            .map(|source| match &source.location {
                OverrideLocation::Archive {
                    archive_path,
                    subdir,
                } => {
                    assert_eq!(archive_path.to_path_buf(), Path::new("cache/pack.mrpack"));
                    (subdir.as_str(), source.side)
                }
                OverrideLocation::Directory { .. } => panic!("an archive's: {source:?}"),
            })
            .collect();
        assert_eq!(
            planned,
            [
                ("overrides", FileSide::Both),
                ("client-overrides", FileSide::Client),
                ("server-overrides", FileSide::Server),
            ]
        );
    }

    #[test]
    fn should_type_files_by_their_top_folder_when_planning_overrides() {
        let files = plan_files(&index(vec![
            index_file("resourcepacks/faithful.zip", &[SODIUM_URL], None),
            index_file("shaderpacks/bsl.zip", &[SODIUM_URL], None),
            index_file("config/tool.jar", &[SODIUM_URL], None),
        ]))
        .unwrap();

        let types: Vec<_> = files.iter().map(|file| file.content_type).collect();
        assert_eq!(
            types,
            [
                ContentType::ResourcePack,
                ContentType::ShaderPack,
                ContentType::Mod
            ]
        );
    }

    #[test]
    fn should_describe_the_pack_when_given_its_index() {
        let metadata = pack_metadata(
            &index(vec![index_file("mods/a.jar", &[SODIUM_URL], None)]),
            None,
            None,
        )
        .unwrap();

        assert_eq!(metadata.name, "Pack");
        assert_eq!(metadata.version_id, None);
        assert_eq!(metadata.version_name.as_deref(), Some("1.4.2"));
        assert_eq!(metadata.game_version, "1.21.1");
        assert_eq!(metadata.mod_loader, ModLoader::NeoForge);
        assert_eq!(metadata.loader_version.as_deref(), Some("21.1.77"));
        assert_eq!(metadata.file_count, Some(1));
    }

    #[test]
    fn should_refuse_the_pack_when_its_game_version_is_not_a_valid_id() {
        let mut index = index(Vec::new());
        index.dependencies.minecraft = "../../escape".into();

        let err = pack_metadata(&index, None, None).unwrap_err();

        assert!(
            matches!(err, InstanceError::InvalidGameVersion { ref value, .. } if value == "../../escape"),
            "{err:?}"
        );
    }

    #[test]
    fn should_refuse_the_pack_when_its_loader_version_is_not_a_valid_id() {
        let mut index = index(Vec::new());
        index.dependencies.loaders = HashMap::from([("neoforge".into(), "21.1 .77".into())]);

        let err = pack_metadata(&index, None, None).unwrap_err();

        assert!(
            matches!(err, InstanceError::InvalidLoaderVersion { ref value, .. } if value == "21.1 .77"),
            "{err:?}"
        );
    }
}
