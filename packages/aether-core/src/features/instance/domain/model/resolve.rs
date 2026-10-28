//! What a provider is asked and answers while an [`InstallPlan`](super::InstallPlan) is put
//! together: the input of `resolve_install_plan`, the input and output of
//! `resolve_pack_metadata`, and the output of `resolve_file`.
//!
//! The ids are the platform's, in the format of [`ContentSource::ProviderRef`]. Which provider
//! gets asked is decided before any of these values is built, so none of them names a provider.

use std::path::PathBuf;

use crate::{
    features::{
        instance::{InstanceId, NewInstance, PackInfo},
        minecraft::{LoaderVersionPreference, ModLoader},
    },
    shared::hash::FileHash,
};

use super::{ContentSource, InstallTarget, InstanceGame};

/// The input of `resolve_install_plan`: which content to install, and into what.
///
/// One shape for a single mod and a whole modpack, installing and updating alike.
///
/// Deliberately no selection of optional files: a provider returns every file, optional ones
/// marked, so the selection UI can be built from the plan; leaving files out is the core's call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveInstallPlanParams {
    pub content_id: String,
    /// `None` — the version [`select_version`](super::select_version) picks among those
    /// compatible with the target.
    pub version: Option<String>,
    /// The platform's id of the version of this content installed in the target now, if any —
    /// a mod's from its pack-file record, a pack's from the instance's `pack_info`. Without a
    /// `version` it sets the release channel and keeps an update from rolling back.
    pub installed_version: Option<String>,
    pub target: ResolveTarget,
}

/// Where the resolved plan is going to be applied. The provider answers an `Existing` request
/// with an `Existing` plan for the same instance, and a `New` request with a `New` plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveTarget {
    /// An existing instance, with the game version and loader it runs now. For any content but
    /// the instance's own pack they are a requirement and the plan keeps the game; for an update
    /// of that pack they are only the current state, and the plan may name a new game.
    Existing {
        instance_id: InstanceId,
        game_version: String,
        /// The loader as the provider's `resolve_file` takes it (`fabric`, `neoforge`, …);
        /// `None` for a vanilla instance.
        loader: Option<String>,
    },
    /// A new instance, described by the content (a modpack): what it runs is the provider's
    /// answer, not a constraint on it.
    New,
}

impl ResolveTarget {
    /// The plan target answering this request for content that fits the instance's game (a
    /// mod): an existing instance keeps it. `new_instance` is asked only for
    /// [`ResolveTarget::New`], where the provider describes the instance from the content.
    pub fn into_install_target(self, new_instance: impl FnOnce() -> NewInstance) -> InstallTarget {
        match self {
            Self::Existing { instance_id, .. } => InstallTarget::existing(instance_id),
            Self::New => InstallTarget::New(new_instance()),
        }
    }

    /// The plan target answering this request for a pack described by `metadata`: a new instance
    /// running the pack's game, or — an update — the existing instance moved to it. `pack_info`
    /// is asked only for [`ResolveTarget::New`], to link the new instance to the pack.
    pub fn into_pack_install_target(
        self,
        metadata: &PackMetadata,
        pack_info: impl FnOnce() -> Option<PackInfo>,
    ) -> InstallTarget {
        match self {
            Self::Existing { instance_id, .. } => InstallTarget::Existing {
                instance_id,
                game: Some(metadata.game()),
            },
            Self::New => InstallTarget::New(metadata.new_instance(pack_info())),
        }
    }

    /// Whether `target` is an answer to this request: the same existing instance, or a new one.
    /// A provider is not trusted with this: a plan for another instance would install into it.
    /// The game the plan names is checked by the executor, against the instance's pack.
    pub fn is_answered_by(&self, target: &InstallTarget) -> bool {
        match (self, target) {
            (
                Self::Existing { instance_id, .. },
                InstallTarget::Existing {
                    instance_id: planned,
                    ..
                },
            ) => instance_id == planned,
            (Self::New, InstallTarget::New(_)) => true,
            _ => false,
        }
    }
}

/// The input of `resolve_pack_metadata`: where to read a pack from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackSource {
    /// A pack hosted on the provider's platform. `version: None` — the version an update would
    /// take ([`select_version`](super::select_version) from `installed_version`), which is how a
    /// check for updates asks.
    Content {
        content_id: String,
        version: Option<String>,
        /// The version of the pack the instance runs now (its `pack_info.version_id`).
        installed_version: Option<String>,
    },
    /// A pack file to download (a `.mrpack`, a packwiz `pack.toml`).
    Url(String),
    /// A pack file on this machine. A host path: anything a plugin names is translated at the
    /// plugin boundary, as for [`OverrideSource`](super::OverrideSource).
    File(PathBuf),
}

/// A pack's description, read without resolving its files: enough to preview it before
/// installing, to create the instance it needs, and to tell whether an update is out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackMetadata {
    pub name: String,
    /// The platform's id of this version, comparable with [`PackInfo::version_id`] — this is what
    /// a check for updates compares. `None` when the pack was not read from the platform
    /// ([`PackSource::Url`], [`PackSource::File`]) and the provider cannot tell.
    pub version_id: Option<String>,
    /// The version as the pack's author wrote it (`1.4.2`), for display only.
    pub version_name: Option<String>,
    pub description: Option<String>,
    pub game_version: String,
    pub mod_loader: ModLoader,
    /// `None` — the pack does not pin one, or the provider cannot tell without reading the pack
    /// itself (a preview from the platform's API).
    pub loader_version: Option<String>,
    /// How many files installing the pack would bring, when known without resolving them
    /// (a packwiz preview reads only `pack.toml`, not its index).
    pub file_count: Option<usize>,
    pub icon_url: Option<String>,
}

impl PackMetadata {
    /// The game this pack runs on; its loader version pinned exactly, when the pack pins one.
    pub fn game(&self) -> InstanceGame {
        InstanceGame {
            game_version: self.game_version.clone(),
            mod_loader: self.mod_loader,
            loader_version: self
                .loader_version
                .clone()
                .map(LoaderVersionPreference::Exact),
        }
    }

    /// The instance this pack runs in, named after the pack. `pack_info` links the instance to
    /// the pack it was installed from, for updates.
    pub fn new_instance(&self, pack_info: Option<PackInfo>) -> NewInstance {
        let InstanceGame {
            game_version,
            mod_loader,
            loader_version,
        } = self.game();

        NewInstance {
            name: self.name.clone(),
            game_version,
            mod_loader,
            loader_version,
            icon_path: None,
            skip_install_instance: None,
            pack_info,
        }
    }
}

/// The answer of `resolve_file`: which bytes a [`ContentSource::ProviderRef`] stands for.
///
/// No path: resolving a ref never moves the file. `file_name` is the platform's name for it, for
/// a caller building a plan entry from scratch and for messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFile {
    /// The platform's id of the version the ref resolved to: for a ref without one, the only
    /// place the core learns which file it got.
    pub version: String,
    pub url: String,
    /// With a hash, the download is verified against it, as for [`ContentSource::Direct`].
    pub hash: Option<FileHash>,
    pub file_name: String,
    /// Size in bytes, when the platform reports it. Informational, like
    /// [`InstallFileEntry::size`](super::InstallFileEntry::size).
    pub size: Option<u64>,
    /// What the platform calls this file (Modrinth: the version's name), recorded with it to show
    /// in the instance's content list when the plan entry has no name of its own.
    pub name: Option<String>,
}

impl ResolvedFile {
    /// What to download in place of the ref: the URL and the hash to check it against.
    pub fn to_source(&self) -> ContentSource {
        ContentSource::direct(self.url.clone(), self.hash.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::features::instance::{
        ContentType, FileSide, InstallFileEntry, InstallPlan, OverrideLocation, OverrideSource,
        ProviderId,
    };

    use super::*;

    const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

    fn modrinth() -> ProviderId {
        ProviderId {
            plugin_id: ProviderId::BUILTIN_PLUGIN_ID.into(),
            capability_id: "modrinth".into(),
        }
    }

    fn sodium_ref(version: Option<&str>) -> ContentSource {
        ContentSource::ProviderRef {
            platform: "modrinth".into(),
            content_id: "AANobbMI".into(),
            version: version.map(Into::into),
        }
    }

    /// A single mod into an existing instance: the request carries the instance's game version
    /// and loader, the plan names the same instance, and the ref in it resolves to a download.
    #[test]
    fn should_build_a_single_mod_install_when_the_instance_exists() {
        let params = ResolveInstallPlanParams {
            content_id: "AANobbMI".into(),
            version: None,
            installed_version: None,
            target: ResolveTarget::Existing {
                instance_id: InstanceId::try_from("instance").unwrap(),
                game_version: "1.21.1".into(),
                loader: Some("fabric".into()),
            },
        };

        let plan = InstallPlan {
            target: params
                .target
                .clone()
                .into_install_target(|| unreachable!("an existing target needs no new instance")),
            files: vec![InstallFileEntry::new(
                sodium_ref(params.version.as_deref()),
                "mods/sodium-fabric-0.6.0.jar",
                ContentType::Mod,
            )],
            overrides: Vec::new(),
        };
        assert_eq!(
            plan.target,
            InstallTarget::existing(InstanceId::try_from("instance").unwrap())
        );

        let resolved = ResolvedFile {
            version: "IZskON6d".into(),
            url: "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d/sodium.jar".into(),
            hash: Some(FileHash::Sha1(SHA1_ABC.into())),
            file_name: "sodium.jar".into(),
            size: Some(1024),
            name: None,
        };
        let entry = InstallFileEntry {
            source: resolved.to_source(),
            ..plan.files[0].clone()
        };

        assert_eq!(
            entry.source,
            ContentSource::direct(resolved.url.clone(), resolved.hash.clone())
        );
        // The platform's file name does not move the file the plan placed.
        assert_eq!(entry.relative_path, "mods/sodium-fabric-0.6.0.jar");
    }

    /// A modpack into a new instance: the preview reads the pack from the platform, and the plan
    /// creates the instance the pack describes, linked back to the pack for updates.
    #[test]
    fn should_build_a_modpack_install_when_the_instance_is_new() {
        let source = PackSource::Content {
            content_id: "1KVo5zza".into(),
            version: None,
            installed_version: None,
        };
        let metadata = PackMetadata {
            name: "Fabulously Optimized".into(),
            version_id: Some("Vq3Rr4wN".into()),
            version_name: Some("6.4.0".into()),
            description: None,
            game_version: "1.21.1".into(),
            mod_loader: ModLoader::Fabric,
            loader_version: Some("0.16.5".into()),
            file_count: Some(2),
            icon_url: None,
        };
        let PackSource::Content { content_id, .. } = &source else {
            unreachable!()
        };

        let params = ResolveInstallPlanParams {
            content_id: content_id.clone(),
            version: metadata.version_id.clone(),
            installed_version: None,
            target: ResolveTarget::New,
        };
        let pack_info = PackInfo {
            provider_id: modrinth(),
            modpack_id: params.content_id.clone(),
            version_id: params.version.clone().unwrap(),
        };

        let plan = InstallPlan {
            target: params
                .target
                .into_install_target(|| metadata.new_instance(Some(pack_info.clone()))),
            files: vec![
                InstallFileEntry::new(
                    sodium_ref(Some("IZskON6d")),
                    "mods/sodium.jar",
                    ContentType::Mod,
                ),
                InstallFileEntry::new(
                    ContentSource::direct(
                        "https://cdn.modrinth.com/data/YL57xq9U/versions/x/iris.jar".into(),
                        None,
                    ),
                    "mods/iris.jar",
                    ContentType::Mod,
                ),
            ],
            overrides: vec![OverrideSource::new(
                OverrideLocation::Archive {
                    archive_path: "cache/fo.mrpack".into(),
                    subdir: "overrides".into(),
                },
                FileSide::Both,
            )],
        };

        let InstallTarget::New(instance) = &plan.target else {
            panic!("a modpack plan creates its instance: {:?}", plan.target)
        };
        assert_eq!(instance.name, "Fabulously Optimized");
        assert_eq!(instance.game_version, "1.21.1");
        assert_eq!(instance.mod_loader, ModLoader::Fabric);
        assert_eq!(
            instance.loader_version,
            Some(LoaderVersionPreference::Exact("0.16.5".into()))
        );
        assert_eq!(instance.pack_info, Some(pack_info));
        assert_eq!(instance.icon_path, None);
        assert_eq!(instance.skip_install_instance, None);
    }

    /// An update of a pack names the pack's game for the instance; a new install creates the
    /// instance on it.
    #[test]
    fn should_answer_a_pack_request_with_the_pack_game_when_resolving() {
        let metadata = PackMetadata {
            name: "Fabulously Optimized".into(),
            version_id: Some("Vq3Rr4wN".into()),
            version_name: None,
            description: None,
            game_version: "1.21.4".into(),
            mod_loader: ModLoader::Fabric,
            loader_version: Some("0.16.9".into()),
            file_count: None,
            icon_url: None,
        };
        let existing = ResolveTarget::Existing {
            instance_id: InstanceId::try_from("instance").unwrap(),
            game_version: "1.21.1".into(),
            loader: Some("fabric".into()),
        };

        assert_eq!(
            existing.into_pack_install_target(&metadata, || unreachable!()),
            InstallTarget::Existing {
                instance_id: InstanceId::try_from("instance").unwrap(),
                game: Some(InstanceGame {
                    game_version: "1.21.4".into(),
                    mod_loader: ModLoader::Fabric,
                    loader_version: Some(LoaderVersionPreference::Exact("0.16.9".into())),
                }),
            }
        );
        assert_eq!(
            ResolveTarget::New.into_pack_install_target(&metadata, || None),
            InstallTarget::New(metadata.new_instance(None))
        );
    }

    #[test]
    fn should_leave_the_loader_version_to_the_default_when_the_pack_does_not_pin_one() {
        let metadata = PackMetadata {
            name: "Vanilla+".into(),
            version_id: None,
            version_name: None,
            description: None,
            game_version: "1.21.1".into(),
            mod_loader: ModLoader::Vanilla,
            loader_version: None,
            file_count: None,
            icon_url: None,
        };

        let instance = metadata.new_instance(None);

        assert_eq!(instance.loader_version, None);
        assert_eq!(instance.pack_info, None);
    }

    #[test]
    fn should_keep_a_resolved_file_unhashed_when_the_provider_gave_no_hash() {
        let resolved = ResolvedFile {
            version: "4512345".into(),
            url: "https://edge.forgecdn.net/files/4512/345/jei.jar".into(),
            hash: None,
            file_name: "jei.jar".into(),
            size: None,
            name: None,
        };

        assert_eq!(
            resolved.to_source(),
            ContentSource::direct(resolved.url.clone(), None)
        );
    }

    #[test]
    fn should_accept_only_a_plan_for_the_requested_target_when_resolving() {
        let existing = ResolveTarget::Existing {
            instance_id: InstanceId::try_from("instance").unwrap(),
            game_version: "1.21.1".into(),
            loader: None,
        };
        let same = InstallTarget::existing(InstanceId::try_from("instance").unwrap());
        // The game a plan names is the executor's to check, against the instance's pack.
        let same_with_game = InstallTarget::Existing {
            instance_id: InstanceId::try_from("instance").unwrap(),
            game: Some(InstanceGame {
                game_version: "1.21.4".into(),
                mod_loader: ModLoader::Fabric,
                loader_version: None,
            }),
        };
        let other = InstallTarget::existing(InstanceId::try_from("other").unwrap());
        let new = InstallTarget::New(NewInstance {
            name: "Pack".into(),
            game_version: "1.21.1".into(),
            mod_loader: ModLoader::Fabric,
            loader_version: None,
            icon_path: None,
            skip_install_instance: None,
            pack_info: None,
        });

        assert!(existing.is_answered_by(&same));
        assert!(existing.is_answered_by(&same_with_game));
        assert!(!existing.is_answered_by(&other));
        assert!(!existing.is_answered_by(&new));
        assert!(ResolveTarget::New.is_answered_by(&new));
        assert!(!ResolveTarget::New.is_answered_by(&same));
    }
}
