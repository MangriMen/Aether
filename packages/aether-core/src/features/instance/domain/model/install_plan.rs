//! The install plan: what a provider wants an instance to contain, described as data and
//! carried out by the core.
//!
//! One contract covers a single mod and a whole modpack, installing and updating alike; only
//! [`InstallTarget`] differs. A provider never writes into the instance: it hands the core the
//! full list of files, and the core decides what to download, verify, write and remove.
//!
//! [`InstallFileEntry`] is a file the plan wants, [`PackFile`](super::PackFile) one the pack
//! storage has recorded, and [`ContentFile`](super::ContentFile) one on disk. Which recorded files
//! a plan may replace or remove is told by
//! [`PackFile::installed_by`](super::PackFile::installed_by), the [`PlanOrigin`] that installed it.

use std::{fmt, path::PathBuf};

use crate::{
    features::{
        instance::{
            ContentMetadata, ContentType, ContentVersionMetadata, Instance, InstanceId,
            NewInstance, ProviderId,
        },
        minecraft::{LoaderVersionPreference, ModLoader},
    },
    shared::hash::FileHash,
};

/// Everything a provider wants installed, and where.
#[derive(Debug, Clone, PartialEq)]
pub struct InstallPlan {
    pub target: InstallTarget,
    pub files: Vec<InstallFileEntry>,
    /// Applied in order, a later source's file replacing an earlier one's at the same path.
    pub overrides: Vec<OverrideSource>,
}

/// Whose plan an [`InstallPlan`] is: the provider that returned it, and the content it was
/// resolved for (`ResolveInstallPlanParams::content_id` — a mod, or a modpack, as
/// [`PackInfo::modpack_id`](super::PackInfo) names it).
///
/// A plan is the whole set of files of its content, so this is also what the files it installs
/// are recorded under ([`PackFile::installed_by`](super::PackFile::installed_by)): the next plan
/// of the same origin replaces that set and leaves every other file in the instance alone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanOrigin {
    /// Also whose word the plan's direct URLs are: they are downloaded under its network policy.
    pub provider_id: ProviderId,
    pub content_id: String,
}

/// The instance a plan applies to. This is the whole difference between install and update:
/// a new instance gets every file, an existing one only the delta against what it has.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallTarget {
    Existing {
        instance_id: InstanceId,
        /// The game the instance runs once the plan is applied; `None` keeps its game.
        ///
        /// `Some` is accepted only from a plan of the pack the instance was installed from (its
        /// [`PackInfo`](super::PackInfo) names the plan's [`PlanOrigin`]): a plan of a single mod
        /// must not move an instance to another game. The plan's refs are resolved against it.
        game: Option<InstanceGame>,
    },
    New(NewInstance),
}

impl InstallTarget {
    /// An existing instance, keeping its game.
    pub fn existing(instance_id: InstanceId) -> Self {
        Self::Existing {
            instance_id,
            game: None,
        }
    }
}

/// What an instance runs: the game version, the loader and its version — as a
/// [`NewInstance`] describes them, and as a pack update may change them.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceGame {
    pub game_version: String,
    pub mod_loader: ModLoader,
    /// `None` — not pinned: the instance keeps its loader version while the game version and
    /// loader stay, and gets the default one for a new game or loader, as a new instance does.
    pub loader_version: Option<LoaderVersionPreference>,
}

impl InstanceGame {
    /// Whether `instance` already runs this game, so moving it here changes nothing.
    pub fn is_run_by(&self, instance: &Instance) -> bool {
        instance.game_version == self.game_version
            && instance.loader == self.mod_loader
            && (self.mod_loader == ModLoader::Vanilla
                || self.loader_version.is_none()
                || instance.loader_version == self.loader_version)
    }
}

/// One file the plan wants in the instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallFileEntry {
    pub source: ContentSource,
    /// Where the file goes, relative to the instance directory, `/`-separated whatever the
    /// host. Resolving a [`ContentSource::ProviderRef`] never changes it.
    ///
    /// Not checked to stay inside the instance here: that happens where the file is written
    /// (`validate_relative_path`), as for every other write.
    pub relative_path: String,
    pub content_type: ContentType,
    /// `None` — the file is required. `Some` — the user may leave it out, and the plan carries
    /// everything the selection UI needs to offer that choice.
    pub optional: Option<OptionalFile>,
    /// The provider reports the side and never drops files itself; skipping a side is the
    /// core's decision.
    pub side: FileSide,
    /// Expected size in bytes, when the provider knows it. Informational (progress); the hash is
    /// what is verified.
    pub size: Option<u64>,
    /// Shown in the instance's content list instead of the file name. For a ref without one,
    /// the name its resolution gives is taken.
    pub name: Option<String>,
    /// What the provider says about the project of the [`ContentSource::Direct`] identity, when
    /// it already knows it: the core caches it at install time, so the content list shows a
    /// fresh install at once. The provider may give it incomplete (without an author, say): it
    /// is cached as stale, and the next list asks for the rest. `None` otherwise; never crosses
    /// the plugin boundary.
    pub metadata: Option<ContentMetadata>,
    /// The same for the version of that identity.
    pub version_metadata: Option<ContentVersionMetadata>,
}

impl InstallFileEntry {
    /// A required entry for both sides, of unknown size and without a name or metadata, its path
    /// normalised.
    pub fn new(source: ContentSource, relative_path: &str, content_type: ContentType) -> Self {
        Self {
            source,
            relative_path: Self::normalize_relative_path(relative_path),
            content_type,
            optional: None,
            side: FileSide::default(),
            size: None,
            name: None,
            metadata: None,
            version_metadata: None,
        }
    }

    /// `relative_path` with every `\` turned into `/`.
    ///
    /// Done on the string, not with `path_slash`: that uses the host's path parser, and on Linux
    /// a plugin's `mods\a.jar` would stay one segment.
    pub fn normalize_relative_path(relative_path: &str) -> String {
        relative_path.replace('\\', "/")
    }

    /// A path as the file system tells paths apart: without case on Windows and macOS, exactly
    /// elsewhere. Two paths with the same key are the same file there.
    pub fn file_system_key(path: &str) -> String {
        Self::path_key(path, cfg!(any(windows, target_os = "macos")))
    }

    /// [`Self::file_system_key`] for a file system that does or does not ignore case.
    pub fn path_key(path: &str, ignores_case: bool) -> String {
        if ignores_case {
            path.to_lowercase()
        } else {
            path.to_owned()
        }
    }
}

/// What makes an [`InstallFileEntry`] optional: whether it is selected unless the user changes
/// that, and what to tell them about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalFile {
    pub default_enabled: bool,
    pub description: Option<String>,
}

/// The side of the game an [`InstallFileEntry`] is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileSide {
    #[default]
    Both,
    Client,
    Server,
}

impl FileSide {
    /// As the pack storage keeps it (the spelling of packwiz's `side`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Both => "both",
            Self::Client => "client",
            Self::Server => "server",
        }
    }
}

/// Where the bytes of an [`InstallFileEntry`] come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentSource {
    /// A URL to download. With a hash, the download is verified against it.
    ///
    /// `identity` — which content of a platform the file is, when the provider knows it (a
    /// Modrinth CDN URL names the project and the version). The file is then recorded for
    /// updates as a resolved ref would be; without it the file is only bytes at a URL.
    Direct {
        url: String,
        hash: Option<FileHash>,
        identity: Option<ContentIdentity>,
    },

    /// Content hosted on a platform, resolved by the core through a provider serving that
    /// platform (`resolve_file`) before downloading. Lets one provider point at content it does
    /// not host without knowing which plugin serves the platform.
    ///
    /// * `platform` — the platform's logical name (`modrinth`, `curseforge`,
    ///   `[a-z0-9][a-z0-9_-]*`) as a provider declares it in
    ///   [`platform`](super::ContentProviderCapabilityMetadata::platform), not a
    ///   [`ProviderId`](super::ProviderId). The core picks the provider; a platform no provider
    ///   serves is an error, never a fallback.
    /// * `content_id` — the platform's stable project id, as in
    ///   [`ContentFileUpdateInfo::content_id`](super::ContentFileUpdateInfo) (Modrinth
    ///   `AANobbMI`, `CurseForge` `238222`).
    /// * `version` — the platform's id of one file of the project (Modrinth `IZskON6d`,
    ///   `CurseForge` `4512345`); `None` asks for the newest compatible file.
    ///
    /// Both ids are the platform's, so every provider of it must accept them unchanged and
    /// swapping the provider never invalidates a ref. The core treats them as opaque and only
    /// compares them for equality.
    ProviderRef {
        platform: String,
        content_id: String,
        version: Option<String>,
    },
}

impl ContentSource {
    /// A direct source with no known identity.
    pub fn direct(url: String, hash: Option<FileHash>) -> Self {
        Self::Direct {
            url,
            hash,
            identity: None,
        }
    }
}

/// One concrete file of a platform's content, in the ids of [`ContentSource::ProviderRef`]
/// (same format and rules) — except that the version is always known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentIdentity {
    pub platform: String,
    pub content_id: String,
    pub version: String,
}

/// Human-readable, for logs and error messages. Not a format to parse back.
impl fmt::Display for ContentSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Direct {
                url, hash: None, ..
            } => f.write_str(url),
            Self::Direct {
                url,
                hash: Some(hash),
                ..
            } => write!(f, "{url} ({hash})"),
            Self::ProviderRef {
                platform,
                content_id,
                version,
            } => write!(
                f,
                "{platform}/{content_id}@{}",
                version.as_deref().unwrap_or("latest")
            ),
        }
    }
}

/// Files copied into the instance as they are, on top of the planned ones (a modpack's
/// `overrides/`, `client-overrides/`).
///
/// `side` is read as [`InstallFileEntry::side`]: the provider reports it and keeps the source in
/// the plan, the core leaves out what is not for its side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideSource {
    pub location: OverrideLocation,
    pub side: FileSide,
}

impl OverrideSource {
    pub fn new(location: OverrideLocation, side: FileSide) -> Self {
        Self { location, side }
    }
}

/// Where the files of an [`OverrideSource`] are. Paths are host paths: anything a plugin names
/// is translated at the plugin boundary, before it gets here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverrideLocation {
    /// The `subdir` directory inside a zip archive.
    Archive {
        archive_path: PathBuf,
        subdir: String,
    },
    Directory {
        path: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

    fn provider_ref(platform: &str, content_id: &str, version: Option<&str>) -> ContentSource {
        ContentSource::ProviderRef {
            platform: platform.into(),
            content_id: content_id.into(),
            version: version.map(Into::into),
        }
    }

    #[test]
    fn should_build_a_direct_source_when_given_manifest_strings() {
        // A packwiz index entry: `hash-format = "sha1"`, `hash = "<HEX>"`.
        let source = ContentSource::direct(
            "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d/sodium.jar".into(),
            Some(FileHash::parse("sha1", &SHA1_ABC.to_ascii_uppercase()).unwrap()),
        );

        assert_eq!(
            source,
            ContentSource::direct(
                "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d/sodium.jar".into(),
                Some(FileHash::Sha1(SHA1_ABC.into()))
            )
        );
    }

    #[test]
    fn should_display_a_direct_source_when_formatted() {
        let url = "https://example.com/a.jar";
        assert_eq!(ContentSource::direct(url.into(), None).to_string(), url);
        assert_eq!(
            ContentSource::direct(url.into(), Some(FileHash::Sha1(SHA1_ABC.into()))).to_string(),
            format!("{url} (sha1:{SHA1_ABC})")
        );
    }

    #[test]
    fn should_display_a_provider_ref_when_formatted() {
        assert_eq!(
            provider_ref("curseforge", "238222", Some("4512345")).to_string(),
            "curseforge/238222@4512345"
        );
        assert_eq!(
            provider_ref("modrinth", "AANobbMI", None).to_string(),
            "modrinth/AANobbMI@latest"
        );
    }

    fn fabric_instance(loader_version: Option<&str>) -> Instance {
        crate::features::instance::InstanceBuilder::new(
            crate::features::instance::InstanceId::try_from("instance").unwrap(),
            "Instance".into(),
            "1.21.1".into(),
            ModLoader::Fabric,
        )
        .with_loader_version(loader_version.map(|v| LoaderVersionPreference::Exact(v.into())))
        .build()
    }

    fn game(
        game_version: &str,
        mod_loader: ModLoader,
        loader_version: Option<&str>,
    ) -> InstanceGame {
        InstanceGame {
            game_version: game_version.into(),
            mod_loader,
            loader_version: loader_version.map(|v| LoaderVersionPreference::Exact(v.into())),
        }
    }

    #[test]
    fn should_tell_whether_an_instance_runs_a_game_when_game_and_loader_are_compared() {
        let instance = fabric_instance(Some("0.16.5"));

        assert!(game("1.21.1", ModLoader::Fabric, Some("0.16.5")).is_run_by(&instance));
        // Not pinned: whatever loader version the instance has will do.
        assert!(game("1.21.1", ModLoader::Fabric, None).is_run_by(&instance));
        assert!(!game("1.21.1", ModLoader::Fabric, Some("0.16.9")).is_run_by(&instance));
        assert!(!game("1.21.4", ModLoader::Fabric, Some("0.16.5")).is_run_by(&instance));
        assert!(!game("1.21.1", ModLoader::Quilt, Some("0.16.5")).is_run_by(&instance));
    }

    #[test]
    fn should_ignore_the_loader_version_when_the_game_is_vanilla() {
        let instance = crate::features::instance::InstanceBuilder::new(
            crate::features::instance::InstanceId::try_from("instance").unwrap(),
            "Instance".into(),
            "1.21.1".into(),
            ModLoader::Vanilla,
        )
        .build();

        assert!(game("1.21.1", ModLoader::Vanilla, Some("0.16.5")).is_run_by(&instance));
    }

    #[test]
    fn should_normalize_backslashes_when_the_host_is_not_windows() {
        let entry = InstallFileEntry::new(
            provider_ref("modrinth", "AANobbMI", None),
            "mods\\sodium.jar",
            ContentType::Mod,
        );

        assert_eq!(entry.relative_path, "mods/sodium.jar");
    }

    #[test]
    fn should_leave_a_slash_path_as_is_when_it_has_no_backslashes() {
        assert_eq!(
            InstallFileEntry::normalize_relative_path("config/sodium/options.json"),
            "config/sodium/options.json"
        );
    }
}
