//! Plugin → core mapping of what `resolveInstallPlan` returns. Only the parts that need no
//! context; the plan's target is checked against the request and its override paths are
//! translated in `PluginContentProviderProxy`, which knows the request and the plugin.

use aether_core_plugin_api::v0::{
    ContentIdentityDto, ContentSourceDto, FileSideDto, InstallFileEntryDto, InstallTargetDto,
    InstanceGameDto, OptionalFileDto,
};

use crate::{
    features::instance::{
        ContentIdentity, ContentSource, FileSide, InstallFileEntry, InstallTarget, InstanceError,
        InstanceGame, InstanceId, OptionalFile,
    },
    shared::hash::{FileHash, InvalidFileHash},
};

/// The id a plugin names is parsed here: one that is not a plain directory name names no
/// instance ([`InstanceError::NotFound`]).
impl TryFrom<InstallTargetDto> for InstallTarget {
    type Error = InstanceError;

    fn try_from(value: InstallTargetDto) -> Result<Self, Self::Error> {
        Ok(match value {
            InstallTargetDto::Existing { instance_id, game } => Self::Existing {
                instance_id: InstanceId::try_from(instance_id)?,
                game: game.map(Into::into),
            },
            InstallTargetDto::New(new_instance) => Self::New(new_instance.into()),
        })
    }
}

impl From<InstanceGameDto> for InstanceGame {
    fn from(value: InstanceGameDto) -> Self {
        Self {
            game_version: value.game_version,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version.map(Into::into),
        }
    }
}

/// A hash from a plugin is validated and lowercased here, as `download_to_cache` does: the
/// core relies on a [`FileHash`] being a well-formed digest.
impl TryFrom<ContentSourceDto> for ContentSource {
    type Error = InvalidFileHash;

    fn try_from(value: ContentSourceDto) -> Result<Self, Self::Error> {
        Ok(match value {
            ContentSourceDto::Direct {
                url,
                hash,
                identity,
            } => Self::Direct {
                url,
                hash: hash.map(FileHash::try_from).transpose()?,
                identity: identity.map(Into::into),
            },
            ContentSourceDto::ProviderRef {
                platform,
                content_id,
                version,
            } => Self::ProviderRef {
                platform,
                content_id,
                version,
            },
        })
    }
}

impl From<ContentIdentityDto> for ContentIdentity {
    fn from(value: ContentIdentityDto) -> Self {
        Self {
            platform: value.platform,
            content_id: value.content_id,
            version: value.version,
        }
    }
}

impl TryFrom<InstallFileEntryDto> for InstallFileEntry {
    type Error = InvalidFileHash;

    fn try_from(value: InstallFileEntryDto) -> Result<Self, Self::Error> {
        Ok(Self {
            optional: value.optional.map(Into::into),
            side: value.side.into(),
            size: value.size,
            name: value.name,
            ..Self::new(
                value.source.try_into()?,
                &value.relative_path,
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
    use aether_core_plugin_api::v0::{ContentTypeDto, FileHashDto};

    use crate::features::instance::ContentType;

    use super::*;

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

        use crate::features::minecraft::{LoaderVersionPreference, ModLoader};

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
                    game_version: "1.21.4".into(),
                    mod_loader: ModLoader::NeoForge,
                    loader_version: Some(LoaderVersionPreference::Exact("21.4.1".into())),
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
                    platform: "modrinth".into(),
                    content_id: "AANobbMI".into(),
                    version: Some("IZskON6d".into()),
                },
                relative_path: "mods/sodium.jar".into(),
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
                url: "https://example.com/a.jar".into(),
                hash: Some(FileHash::parse("sha1", SHA1_ABC).unwrap()),
                identity: Some(ContentIdentity {
                    platform: "modrinth".into(),
                    content_id: "AANobbMI".into(),
                    version: "IZskON6d".into(),
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
}
