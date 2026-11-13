//! Plugin → core mapping of what `resolveInstallPlan` returns. Only the parts that need no
//! context; the plan's target is checked against the request and its override paths are
//! translated in `PluginContentProviderProxy`, which knows the request and the plugin.

use aether_core_plugin_api::v0::{
    ContentIdentityDto, ContentSourceDto, FileSideDto, InstallFileEntryDto, InstallTargetDto,
    InstanceGameDto, OptionalFileDto,
};

use crate::{
    features::{
        instance::{
            ContentIdentity, ContentPath, ContentSource, FileSide, InstallFileEntry, InstallTarget,
            InstanceError, InstanceGame, InstanceId, InvalidContentId, InvalidId,
            InvalidRelativePath, OptionalFile, PlatformId,
        },
        minecraft::GameVersion,
    },
    shared::{
        hash::{FileHash, InvalidFileHash},
        web_url::{HttpsUrl, InvalidUrl},
    },
};

/// The id a plugin names is parsed here: one that is not a plain directory name names no
/// instance ([`InstanceError::NotFound`]).
impl TryFrom<InstallTargetDto> for InstallTarget {
    type Error = InstanceError;

    fn try_from(value: InstallTargetDto) -> Result<Self, Self::Error> {
        Ok(match value {
            InstallTargetDto::Existing { instance_id, game } => Self::Existing {
                instance_id: InstanceId::try_from(instance_id)?,
                game: game.map(TryInto::try_into).transpose()?,
            },
            InstallTargetDto::New(new_instance) => Self::New(new_instance.try_into()?),
        })
    }
}

impl TryFrom<InstanceGameDto> for InstanceGame {
    type Error = InstanceError;

    fn try_from(value: InstanceGameDto) -> Result<Self, Self::Error> {
        Ok(Self {
            game_version: GameVersion::try_from(value.game_version)?,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version.map(TryInto::try_into).transpose()?,
        })
    }
}

/// What is wrong with a file a plugin put in its plan.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPlanFile {
    #[error(transparent)]
    Hash(#[from] InvalidFileHash),

    #[error("Invalid platform: {0}")]
    Platform(#[from] InvalidId),

    #[error(transparent)]
    ContentId(#[from] InvalidContentId),

    #[error(transparent)]
    Path(#[from] InvalidRelativePath),

    #[error(transparent)]
    Url(#[from] InvalidUrl),
}

impl InvalidPlanFile {
    /// The URL refused, when that is what is wrong with the file.
    pub(crate) fn invalid_url(&self) -> Option<&InvalidUrl> {
        match self {
            Self::Url(err) => Some(err),
            _ => None,
        }
    }
}

/// A hash from a plugin is validated and lowercased here, as `download_to_cache` does: the
/// core relies on a [`FileHash`] being a well-formed digest. A URL to download has to be `https`.
impl TryFrom<ContentSourceDto> for ContentSource {
    type Error = InvalidPlanFile;

    fn try_from(value: ContentSourceDto) -> Result<Self, Self::Error> {
        Ok(match value {
            ContentSourceDto::Direct {
                url,
                hash,
                identity,
            } => Self::Direct {
                url: HttpsUrl::try_from(url)?,
                hash: hash.map(FileHash::try_from).transpose()?,
                identity: identity.map(ContentIdentity::try_from).transpose()?,
            },
            ContentSourceDto::ProviderRef {
                platform,
                content_id,
                version,
            } => Self::ProviderRef {
                platform: PlatformId::try_from(platform)?,
                content_id: content_id.try_into()?,
                version: version.map(TryInto::try_into).transpose()?,
            },
        })
    }
}

impl TryFrom<ContentIdentityDto> for ContentIdentity {
    type Error = InvalidPlanFile;

    fn try_from(value: ContentIdentityDto) -> Result<Self, Self::Error> {
        Ok(Self {
            platform: PlatformId::try_from(value.platform)?,
            content_id: value.content_id.try_into()?,
            version: value.version.try_into()?,
        })
    }
}

impl TryFrom<InstallFileEntryDto> for InstallFileEntry {
    type Error = InvalidPlanFile;

    fn try_from(value: InstallFileEntryDto) -> Result<Self, Self::Error> {
        Ok(Self {
            optional: value.optional.map(Into::into),
            side: value.side.into(),
            size: value.size,
            name: value.name,
            ..Self::new(
                value.source.try_into()?,
                ContentPath::parse(&value.relative_path)?,
                value.content_type.into(),
            )
        })
    }
}

impl From<OptionalFileDto> for OptionalFile {
    fn from(value: OptionalFileDto) -> Self {
        Self {
            default_enabled: value.default_enabled,
            description: value.description,
        }
    }
}

impl From<FileSideDto> for FileSide {
    fn from(value: FileSideDto) -> Self {
        match value {
            FileSideDto::Both => Self::Both,
            FileSideDto::Client => Self::Client,
            FileSideDto::Server => Self::Server,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::features::instance::{ContentId, ContentVersionId};
    use aether_core_plugin_api::v0::{ContentTypeDto, FileHashDto};

    use crate::features::instance::ContentType;

    use super::*;
    use crate::shared::hash::HashAlgorithm;

    const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

    fn entry(source: ContentSourceDto) -> InstallFileEntryDto {
        InstallFileEntryDto {
            source,
            relative_path: "mods\\sodium.jar".into(),
            content_type: ContentTypeDto::Mod,
            optional: Some(OptionalFileDto {
                default_enabled: false,
                description: Some("Faster rendering".into()),
            }),
            side: FileSideDto::Client,
            size: Some(42),
            name: Some("Sodium 0.6.0".into()),
        }
    }

    #[test]
    fn should_map_the_game_when_the_target_is_an_existing_instance() {
        use aether_core_plugin_api::v0::{LoaderVersionPreferenceDto, ModLoaderDto};

        use crate::features::minecraft::{LoaderVersionId, LoaderVersionPreference, ModLoader};

        let target = InstallTarget::try_from(InstallTargetDto::Existing {
            instance_id: "instance".into(),
            game: Some(InstanceGameDto {
                game_version: "1.21.4".into(),
                mod_loader: ModLoaderDto::NeoForge,
                loader_version: Some(LoaderVersionPreferenceDto::Exact("21.4.1".into())),
            }),
        })
        .unwrap();

        assert_eq!(
            target,
            InstallTarget::Existing {
                instance_id: InstanceId::try_from("instance").unwrap(),
                game: Some(InstanceGame {
                    game_version: GameVersion::from_static("1.21.4"),
                    mod_loader: ModLoader::NeoForge,
                    loader_version: Some(LoaderVersionPreference::Exact(
                        LoaderVersionId::from_static("21.4.1")
                    )),
                }),
            }
        );
        assert_eq!(
            InstallTarget::try_from(InstallTargetDto::Existing {
                instance_id: "instance".into(),
                game: None,
            })
            .unwrap(),
            InstallTarget::existing(InstanceId::try_from("instance").unwrap())
        );
    }

    #[test]
    fn should_refuse_the_target_when_its_game_version_is_not_a_valid_id() {
        use aether_core_plugin_api::v0::ModLoaderDto;

        let err = InstallTarget::try_from(InstallTargetDto::Existing {
            instance_id: "instance".into(),
            game: Some(InstanceGameDto {
                game_version: "../../x".into(),
                mod_loader: ModLoaderDto::Fabric,
                loader_version: None,
            }),
        })
        .unwrap_err();

        assert!(
            matches!(err, InstanceError::InvalidGameVersion { ref value, .. } if value == "../../x"),
            "{err}"
        );
    }

    #[test]
    fn should_refuse_the_target_when_its_loader_version_is_not_a_valid_id() {
        use aether_core_plugin_api::v0::{
            LoaderVersionPreferenceDto, ModLoaderDto, NewInstanceDto,
        };

        let bad_loader = || Some(LoaderVersionPreferenceDto::Exact("0.16 .5".into()));
        let existing = InstallTarget::try_from(InstallTargetDto::Existing {
            instance_id: "instance".into(),
            game: Some(InstanceGameDto {
                game_version: "1.21.1".into(),
                mod_loader: ModLoaderDto::Fabric,
                loader_version: bad_loader(),
            }),
        })
        .unwrap_err();
        let new = InstallTarget::try_from(InstallTargetDto::New(NewInstanceDto {
            name: "Pack".into(),
            game_version: "1.21.1".into(),
            mod_loader: ModLoaderDto::Fabric,
            loader_version: bad_loader(),
            icon_path: None,
            skip_install_instance: None,
            pack_info: None,
        }))
        .unwrap_err();

        for err in [existing, new] {
            assert!(
                matches!(err, InstanceError::InvalidLoaderVersion { ref value, .. } if value == "0.16 .5"),
                "{err}"
            );
        }
    }

    /// A plugin's plan for `../x` would install next to the instances; it names no instance.
    #[test]
    fn should_refuse_the_target_when_its_instance_id_is_not_a_plain_name() {
        let err = InstallTarget::try_from(InstallTargetDto::Existing {
            instance_id: "../x".into(),
            game: None,
        })
        .unwrap_err();

        assert!(
            matches!(&err, InstanceError::NotFound { instance_id } if instance_id == "../x"),
            "{err}"
        );
    }

    #[test]
    fn should_map_every_field_when_given_an_entry() {
        let entry: InstallFileEntry = entry(ContentSourceDto::ProviderRef {
            platform: "modrinth".into(),
            content_id: "AANobbMI".into(),
            version: Some("IZskON6d".into()),
        })
        .try_into()
        .unwrap();

        assert_eq!(
            entry,
            InstallFileEntry {
                source: ContentSource::ProviderRef {
                    platform: PlatformId::from_static("modrinth"),
                    content_id: ContentId::try_from("AANobbMI").unwrap(),
                    version: Some(ContentVersionId::try_from("IZskON6d").unwrap()),
                },
                relative_path: ContentPath::parse("mods/sodium.jar").unwrap(),
                content_type: ContentType::Mod,
                optional: Some(OptionalFile {
                    default_enabled: false,
                    description: Some("Faster rendering".into()),
                }),
                side: FileSide::Client,
                size: Some(42),
                name: Some("Sodium 0.6.0".into()),
                // The plugin API has no metadata: none comes from a plugin.
                metadata: None,
                version_metadata: None,
            }
        );
    }

    #[test]
    fn should_lowercase_a_direct_hash_and_keep_its_identity_when_mapping() {
        let source: ContentSource = ContentSourceDto::Direct {
            url: "https://example.com/a.jar".into(),
            hash: Some(FileHashDto::Sha1(SHA1_ABC.to_ascii_uppercase())),
            identity: Some(ContentIdentityDto {
                platform: "modrinth".into(),
                content_id: "AANobbMI".into(),
                version: "IZskON6d".into(),
            }),
        }
        .try_into()
        .unwrap();

        assert_eq!(
            source,
            ContentSource::Direct {
                url: HttpsUrl::parse("https://example.com/a.jar").unwrap(),
                hash: Some(FileHash::new(HashAlgorithm::Sha1, SHA1_ABC).unwrap()),
                identity: Some(ContentIdentity {
                    platform: PlatformId::from_static("modrinth"),
                    content_id: ContentId::try_from("AANobbMI").unwrap(),
                    version: ContentVersionId::try_from("IZskON6d").unwrap(),
                }),
            }
        );
    }

    #[test]
    fn should_reject_a_hash_when_it_is_malformed() {
        for hash in [
            FileHashDto::Sha1("abc".into()),
            FileHashDto::Sha256("z".repeat(64)),
        ] {
            let result = InstallFileEntry::try_from(entry(ContentSourceDto::Direct {
                url: "https://example.com/a.jar".into(),
                hash: Some(hash.clone()),
                identity: None,
            }));

            assert!(result.is_err(), "{hash:?} must be refused");
        }
    }

    /// The host downloads a plan's URLs itself: one that is not `https`, or not a URL, fails the
    /// plan instead of being passed on.
    #[test]
    fn should_reject_a_direct_url_when_it_is_not_https() {
        for url in [
            "http://example.com/a.jar",
            "file:///C:/Users/me/a.jar",
            "/cache/a.jar",
            "javascript:alert(1)",
            "https://user:secret@example.com/a.jar",
            "",
        ] {
            let result = InstallFileEntry::try_from(entry(ContentSourceDto::Direct {
                url: url.into(),
                hash: None,
                identity: None,
            }));

            assert!(
                matches!(result, Err(InvalidPlanFile::Url(_))),
                "`{url}` must be refused"
            );
        }
    }

    /// A plan file at `../x` or under `.metadata` is refused when the plan is mapped, before
    /// anything of it is resolved.
    #[test]
    fn should_reject_a_path_when_it_leaves_the_instance_or_is_metadata() {
        for path in [
            "../x",
            "mods/../../x",
            "/abs/x.jar",
            ".metadata/x",
            "",
            "C:/x.jar",
            ".METADATA/x",
        ] {
            let mut dto = entry(ContentSourceDto::Direct {
                url: "https://example.com/a.jar".into(),
                hash: None,
                identity: None,
            });
            dto.relative_path = path.into();

            let result = InstallFileEntry::try_from(dto);

            assert!(
                matches!(result, Err(InvalidPlanFile::Path(_))),
                "`{path}` must be refused"
            );
        }
    }

    #[test]
    fn should_reject_a_ref_when_its_content_id_is_not_a_valid_id() {
        for (content_id, version) in [("a/b", None), ("AANobbMI", Some("v?1")), ("", None)] {
            let dto = entry(ContentSourceDto::ProviderRef {
                platform: "modrinth".into(),
                content_id: content_id.into(),
                version: version.map(Into::into),
            });

            let result = InstallFileEntry::try_from(dto);

            assert!(
                matches!(result, Err(InvalidPlanFile::ContentId(_))),
                "{content_id:?} {version:?} must be refused"
            );
        }
    }
}
