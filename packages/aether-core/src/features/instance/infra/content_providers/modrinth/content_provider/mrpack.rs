//! A `.mrpack` read into plan data: its index becomes the plan's files, its `overrides/` (and
//! those of each side) the plan's overrides, its dependencies the game the pack runs on. Nothing
//! here touches the network: the archive is already on disk.

use std::{io::Read, path::Path};

use url::Url;

use crate::{
    features::instance::{
        ContentIdentity, ContentSource, ContentType, FileSide, InstallFileEntry, InstanceError,
        OptionalFile, OverrideLocation, OverrideSource, PackMetadata,
        infra::content_providers::modrinth::api_client::{
            Hashes, ModrinthEnv, ModrinthIndex, ModrinthIndexFile, ProjectVersionResponse,
        },
    },
    features::minecraft::ModLoader,
    shared::{
        hash::{FileHash, HashAlgorithm},
        limits::{MAX_MRPACK_FILES, MAX_MRPACK_INDEX_BYTES},
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

fn invalid_pack(reason: impl Into<String>) -> InstanceError {
    InstanceError::ContentProviderError {
        reason: reason.into(),
    }
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
        .map_err(|e| invalid_pack(format!("Failed to open .mrpack file: {e}")))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| invalid_pack(format!("Invalid .mrpack archive: {e}")))?;

    let index_file = archive
        .by_name(INDEX_PATH)
        .map_err(|_| invalid_pack(format!("Critical: {INDEX_PATH} missing in modpack")))?;

    // One byte past the limit tells an index that is too large from one that fits exactly.
    let mut bytes = Vec::new();
    index_file
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| invalid_pack(format!("Failed to read {INDEX_PATH}: {e}")))?;
    if bytes.len() as u64 > max_bytes {
        return Err(invalid_pack(format!(
            "{INDEX_PATH} is larger than {max_bytes} bytes"
        )));
    }

    let index: ModrinthIndex = serde_json::from_slice(&bytes)
        .map_err(|e| invalid_pack(format!("Malformed modpack index: {e}")))?;

    if index.files.len() > max_files {
        return Err(invalid_pack(format!(
            "{INDEX_PATH} lists {} files, more than {max_files}",
            index.files.len()
        )));
    }

    if index.game != "minecraft" {
        return Err(invalid_pack(format!(
            "Modpack is for \"{}\", not Minecraft",
            index.game
        )));
    }

    Ok(index)
}

/// [`read_index`] off the async runtime.
pub(super) async fn read_index_async(mrpack_path: &Path) -> Result<ModrinthIndex, InstanceError> {
    let mrpack_path = mrpack_path.to_owned();
    tokio::task::spawn_blocking(move || read_index(&mrpack_path))
        .await
        .map_err(|e| invalid_pack(e.to_string()))?
}

/// The project and version a Modrinth CDN URL names:
/// `https://cdn.modrinth.com/data/<project>/versions/<version>/<file>`.
pub(super) fn parse_cdn_url(url: &str) -> Option<(String, String)> {
    let url = Url::parse(url).ok()?;
    if url.host_str() != Some(MODRINTH_CDN_HOST) {
        return None;
    }

    match url.path_segments()?.collect::<Vec<_>>()[..] {
        ["data", project_id, "versions", version_id, _, ..]
            if !project_id.is_empty() && !version_id.is_empty() =>
        {
            Some((project_id.to_owned(), version_id.to_owned()))
        }
        _ => None,
    }
}

/// Which Modrinth content a URL is, when it is a file on Modrinth's CDN.
pub(super) fn cdn_identity(url: &str) -> Option<ContentIdentity> {
    parse_cdn_url(url).map(|(content_id, version)| ContentIdentity {
        platform: ModrinthContentProvider::PLATFORM.to_owned(),
        content_id,
        version,
    })
}

/// The file's SHA-512, the stronger of the two hashes Modrinth gives, validated.
pub(super) fn sha512_of(hashes: &Hashes) -> Result<FileHash, InstanceError> {
    FileHash::new(HashAlgorithm::Sha512, hashes.sha512.clone())
        .map_err(|e| invalid_pack(format!("Invalid SHA-512 from Modrinth: {e}")))
}

/// Every file of the index as a plan entry — optional and server-only ones too: leaving files
/// out is the core's decision.
pub(super) fn plan_files(index: &ModrinthIndex) -> Result<Vec<InstallFileEntry>, InstanceError> {
    index.files.iter().map(plan_file).collect()
}

/// The override directories of the archive, every side's included: as with the files, leaving
/// out the server's is the core's decision. A directory the archive lacks lists no files.
pub(super) fn plan_overrides(archive_path: &Path) -> Vec<OverrideSource> {
    OVERRIDES_DIRS
        .iter()
        .map(|&(subdir, side)| {
            OverrideSource::new(
                OverrideLocation::Archive {
                    archive_path: archive_path.to_owned(),
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
        .find(|url| is_allowed_download(url))
        .ok_or_else(|| {
            invalid_pack(format!(
                "No download of \"{}\" is on a host the .mrpack format allows",
                file.path
            ))
        })?;

    let (side, optional) = side_and_option(file.env.as_ref());

    Ok(InstallFileEntry {
        optional,
        side,
        size: Some(file.file_size),
        ..InstallFileEntry::new(
            ContentSource::Direct {
                url: url.clone(),
                hash: Some(sha512_of(&file.hashes)?),
                identity: cdn_identity(url),
            },
            &file.path,
            content_type_of(&file.path),
        )
    })
}

fn is_allowed_download(url: &str) -> bool {
    Url::parse(url).is_ok_and(|url| {
        url.scheme() == "https"
            && url
                .host_str()
                .is_some_and(|host| ALLOWED_DOWNLOAD_HOSTS.contains(&host))
    })
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
    version: Option<&ProjectVersionResponse>,
    icon_url: Option<String>,
) -> PackMetadata {
    let (mod_loader, loader_version) = resolve_loader_from_manifest(index);

    PackMetadata {
        name: index.name.clone(),
        version_id: version.map(|version| version.id.clone()),
        version_name: Some(index.version_id.clone()),
        description: index.summary.clone(),
        game_version: index.dependencies.minecraft.clone(),
        mod_loader,
        loader_version,
        file_count: Some(index.files.len()),
        icon_url,
    }
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

        assert!(err.to_string().contains("larger than"), "{err}");
    }

    #[test]
    fn should_refuse_an_index_when_it_lists_too_many_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let files = (0..3)
            .map(|i| index_file(&format!("mods/{i}.jar"), &[], None))
            .collect();
        let path = mrpack_with_index(&dir, &serde_json::to_vec(&index(files)).unwrap());

        let err = read_index_within(&path, MAX_MRPACK_INDEX_BYTES, 2).unwrap_err();

        assert!(err.to_string().contains("more than 2"), "{err}");
    }

    #[test]
    fn should_read_the_project_and_version_when_given_a_cdn_url() {
        assert_eq!(
            parse_cdn_url(SODIUM_URL),
            Some(("AANobbMI".into(), "IZskON6d".into()))
        );
        for url in [
            "https://example.com/data/AANobbMI/versions/IZskON6d/sodium.jar",
            "https://cdn.modrinth.com/data/AANobbMI/sodium.jar",
            "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d",
            "not a url",
        ] {
            assert_eq!(parse_cdn_url(url), None, "{url}");
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
                url: SODIUM_URL.into(),
                hash: Some(FileHash::new(HashAlgorithm::Sha512, "b".repeat(128)).unwrap()),
                identity: Some(ContentIdentity {
                    platform: "modrinth".into(),
                    content_id: "AANobbMI".into(),
                    version: "IZskON6d".into(),
                }),
            }
        );
        assert_eq!(files[0].relative_path, "mods/sodium.jar");
        assert_eq!(files[0].size, Some(42));
        assert!(matches!(
            &files[1].source,
            ContentSource::Direct { url, identity: None, .. } if url == github
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
            ContentSource::Direct { url, .. } if url == SODIUM_URL
        ));
    }

    #[test]
    fn should_refuse_a_file_when_no_download_is_on_an_allowed_host() {
        let result = plan_files(&index(vec![index_file(
            "mods/evil.jar",
            &["https://localhost/evil.jar", "https://example.com/evil.jar"],
            None,
        )]));

        assert!(result.is_err());
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
                    assert_eq!(archive_path, Path::new("cache/pack.mrpack"));
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
        );

        assert_eq!(metadata.name, "Pack");
        assert_eq!(metadata.version_id, None);
        assert_eq!(metadata.version_name.as_deref(), Some("1.4.2"));
        assert_eq!(metadata.game_version, "1.21.1");
        assert_eq!(metadata.mod_loader, ModLoader::NeoForge);
        assert_eq!(metadata.loader_version.as_deref(), Some("21.1.77"));
        assert_eq!(metadata.file_count, Some(1));
    }
}
