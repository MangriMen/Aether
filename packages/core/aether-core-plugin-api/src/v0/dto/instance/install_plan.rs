//! The install plan a content provider returns from its `resolveInstallPlan` handler: what an
//! instance should contain, described as data. The launcher downloads, verifies and writes the
//! files itself; a provider never writes into the instance.

use serde::{Deserialize, Serialize};

use crate::v0::{
    ContentTypeDto, FileHashDto, LoaderVersionPreferenceDto, ModLoaderDto, NewInstanceDto,
};

/// Everything a provider wants installed, and where.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanDto {
    pub target: InstallTargetDto,
    /// Every file the content needs, optional and server-side ones included: leaving files out
    /// is the launcher's decision, made from the plan.
    pub files: Vec<InstallFileEntryDto>,
    /// Applied in order, a later source's file replacing an earlier one's at the same path.
    #[serde(default)]
    pub overrides: Vec<OverrideSourceDto>,
}

/// The instance a plan applies to.
///
/// Answers the request's `ResolveTargetDto`: `existing` with the same instance id, `new` with
/// the instance the content needs (a modpack's game version and loader). A plan for any other
/// instance is rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum InstallTargetDto {
    Existing {
        instance_id: String,
        /// The game the instance runs once the plan is applied. Absent — the instance keeps its
        /// game.
        ///
        /// Only an update of the pack the instance was installed from may set it (its
        /// `packInfo` names the provider and content the plan was resolved for); the request's
        /// `gameVersion` and `loader` are then the instance's current state, not a requirement.
        /// Any other plan with a game is rejected.
        #[serde(default)]
        game: Option<InstanceGameDto>,
    },
    New(NewInstanceDto),
}

/// What an instance runs, as a `NewInstanceDto` describes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceGameDto {
    pub game_version: String,
    pub mod_loader: ModLoaderDto,
    /// Absent — not pinned: the instance keeps its loader version while the game version and
    /// loader stay, and gets the default one otherwise.
    #[serde(default)]
    pub loader_version: Option<LoaderVersionPreferenceDto>,
}

/// One file the plan wants in the instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallFileEntryDto {
    pub source: ContentSourceDto,
    /// Where the file goes, relative to the instance directory (`mods/sodium.jar`). `\` is
    /// read as `/`. A path leaving the instance is refused when the file is written.
    pub relative_path: String,
    pub content_type: ContentTypeDto,
    /// Absent — the file is required.
    #[serde(default)]
    pub optional: Option<OptionalFileDto>,
    /// Absent — the file is for both sides.
    #[serde(default)]
    pub side: FileSideDto,
    /// Expected size in bytes, when known. Informational; the hash is what is verified.
    #[serde(default)]
    pub size: Option<u64>,
    /// What the file is called (a mod's or version's name), shown in the instance's content
    /// list. Absent — the name `resolveFile` gives for a `providerRef`, or none.
    #[serde(default)]
    pub name: Option<String>,
}

/// What makes a file optional: whether it is selected unless the user changes that, and what to
/// tell them about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFileDto {
    pub default_enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
}

/// The side of the game a file is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileSideDto {
    #[default]
    Both,
    Client,
    Server,
}

/// Where the bytes of a file come from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ContentSourceDto {
    /// A URL to download, checked against the plugin's `allowedHosts` like its own requests.
    /// With a hash, the download is verified against it.
    Direct {
        url: String,
        #[serde(default)]
        hash: Option<FileHashDto>,
        /// Which content of a platform the file is, when known (a pack listing a Modrinth CDN
        /// URL). The file is then recorded with it for updates, and a new version of a mod the
        /// user disabled stays disabled under a new file name.
        #[serde(default)]
        identity: Option<ContentIdentityDto>,
    },
    /// Content hosted on a platform, resolved by the launcher through a provider serving that
    /// platform (its `resolveFile`) — how a pack points at content it does not host, without
    /// knowing which plugin serves it.
    ///
    /// `platform` is the platform's name (`modrinth`, `curseforge`), not a plugin id;
    /// `contentId` the platform's project id (Modrinth `AANobbMI`, `CurseForge` `238222`);
    /// `version` the platform's id of one file of it (Modrinth version id, `CurseForge` file id),
    /// absent — the newest file fitting the instance.
    ProviderRef {
        platform: String,
        content_id: String,
        #[serde(default)]
        version: Option<String>,
    },
}

/// One concrete file of a platform's content, in the ids of `ContentSourceDto::ProviderRef`, its
/// version always known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentIdentityDto {
    pub platform: String,
    pub content_id: String,
    pub version: String,
}

/// Files copied into the instance as they are, on top of the planned ones (a modpack's
/// `overrides/`, `client-overrides/`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideSourceDto {
    pub location: OverrideLocationDto,
    /// Absent — for both sides. As with files, a source for the other side stays in the plan:
    /// the launcher leaves it out.
    #[serde(default)]
    pub side: FileSideDto,
}

/// Where the files of an override source are.
///
/// Paths are plugin-side and must lie in the plugin's `/cache` (where `download_to_cache` puts
/// files); the launcher refuses anything else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OverrideLocationDto {
    /// The `subdir` directory of a zip archive (`overrides`).
    Archive {
        archive_path: String,
        subdir: String,
    },
    /// A directory, copied recursively.
    Directory { path: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_default_optional_side_and_size_when_absent() {
        let entry: InstallFileEntryDto = serde_json::from_str(
            r#"{
                "source": { "providerRef": { "platform": "modrinth", "contentId": "AANobbMI" } },
                "relativePath": "mods/sodium.jar",
                "contentType": "mod"
            }"#,
        )
        .unwrap();

        assert_eq!(
            entry,
            InstallFileEntryDto {
                source: ContentSourceDto::ProviderRef {
                    platform: "modrinth".into(),
                    content_id: "AANobbMI".into(),
                    version: None,
                },
                relative_path: "mods/sodium.jar".into(),
                content_type: ContentTypeDto::Mod,
                optional: None,
                side: FileSideDto::Both,
                size: None,
                name: None,
            }
        );
    }

    #[test]
    fn should_keep_the_game_of_an_existing_instance_when_absent() {
        let target: InstallTargetDto =
            serde_json::from_str(r#"{ "existing": { "instanceId": "instance" } }"#).unwrap();

        assert!(matches!(
            target,
            InstallTargetDto::Existing { instance_id, game: None } if instance_id == "instance"
        ));
    }

    #[test]
    fn should_read_the_game_of_an_existing_instance() {
        let target: InstallTargetDto = serde_json::from_str(
            r#"{ "existing": {
                "instanceId": "instance",
                "game": { "gameVersion": "1.21.4", "modLoader": "fabric" }
            } }"#,
        )
        .unwrap();

        let InstallTargetDto::Existing {
            game: Some(game), ..
        } = target
        else {
            panic!("expected a game: {target:?}")
        };
        assert_eq!(game.game_version, "1.21.4");
        assert!(matches!(game.mod_loader, ModLoaderDto::Fabric));
        assert!(game.loader_version.is_none());
    }

    #[test]
    fn should_name_variants_and_their_fields_in_camel_case() {
        let entry = InstallFileEntryDto {
            source: ContentSourceDto::Direct {
                url: "https://example.com/a.jar".into(),
                hash: Some(FileHashDto::Sha1("a".repeat(40))),
                identity: Some(ContentIdentityDto {
                    platform: "modrinth".into(),
                    content_id: "AANobbMI".into(),
                    version: "IZskON6d".into(),
                }),
            },
            relative_path: "mods/a.jar".into(),
            content_type: ContentTypeDto::Mod,
            optional: Some(OptionalFileDto {
                default_enabled: false,
                description: None,
            }),
            side: FileSideDto::Client,
            size: Some(1),
            name: Some("Sodium 0.6.0".into()),
        };
        let overrides = OverrideSourceDto {
            location: OverrideLocationDto::Archive {
                archive_path: "/cache/pack.zip".into(),
                subdir: "client-overrides".into(),
            },
            side: FileSideDto::Client,
        };

        let entry_json = serde_json::to_string(&entry).unwrap();
        let overrides_json = serde_json::to_string(&overrides).unwrap();

        assert!(entry_json.contains(r#""direct":{"url""#), "{entry_json}");
        assert!(
            entry_json.contains(r#""defaultEnabled":false"#),
            "{entry_json}"
        );
        assert!(entry_json.contains(r#""side":"client""#), "{entry_json}");
        assert!(
            entry_json.contains(r#""identity":{"platform":"modrinth","contentId""#),
            "{entry_json}"
        );
        assert!(
            overrides_json.contains(r#""location":{"archive":{"archivePath""#),
            "{overrides_json}"
        );
        assert!(
            overrides_json.contains(r#""side":"client""#),
            "{overrides_json}"
        );
    }

    #[test]
    fn should_read_overrides_in_order_with_both_sides_by_default() {
        let plan: InstallPlanDto = serde_json::from_str(
            r#"{
                "target": { "existing": { "instanceId": "instance" } },
                "files": [],
                "overrides": [
                    { "location": { "archive": { "archivePath": "/cache/p.zip", "subdir": "overrides" } } },
                    { "location": { "directory": { "path": "/cache/client" } }, "side": "client" }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(
            plan.overrides,
            [
                OverrideSourceDto {
                    location: OverrideLocationDto::Archive {
                        archive_path: "/cache/p.zip".into(),
                        subdir: "overrides".into(),
                    },
                    side: FileSideDto::Both,
                },
                OverrideSourceDto {
                    location: OverrideLocationDto::Directory {
                        path: "/cache/client".into(),
                    },
                    side: FileSideDto::Client,
                },
            ]
        );
    }

    #[test]
    fn should_have_no_overrides_when_absent() {
        let plan: InstallPlanDto = serde_json::from_str(
            r#"{ "target": { "existing": { "instanceId": "instance" } }, "files": [] }"#,
        )
        .unwrap();

        assert!(plan.overrides.is_empty());
    }
}
