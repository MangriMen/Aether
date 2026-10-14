//! What the launcher asks a content provider's `resolve*` handlers and what they answer.
//!
//! Ids are the platform's: a `contentId` is the project, a `version` one concrete file of it,
//! and an absent `version` asks for the newest one that fits.

use serde::{Deserialize, Serialize};

use crate::v0::{FileHashDto, ModLoaderDto};

/// The input of `resolveFile`: which file a `providerRef` stands for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveFileParamsDto {
    pub content_id: String,
    /// Absent — the newest file that runs on `gameVersion` with `loader`. With a version, the
    /// provider returns that file and need not check it against them.
    #[serde(default)]
    pub version: Option<String>,
    pub game_version: String,
    /// The loader as written in `ResolveTargetDto::Existing` (`fabric`, `neoforge`); absent —
    /// vanilla.
    #[serde(default)]
    pub loader: Option<String>,
}

/// The answer of `resolveFile`: which bytes the ref stands for. No path: where the file goes is
/// the plan entry's `relativePath`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedFileDto {
    /// The platform's id of the file the ref resolved to.
    pub version: String,
    /// Downloaded under the same rules as a `direct` source of a plan.
    pub url: String,
    #[serde(default)]
    pub hash: Option<FileHashDto>,
    /// The name the platform gives the file.
    pub file_name: String,
    #[serde(default)]
    pub size: Option<u64>,
    /// What the platform calls the file (a version's name), shown in the instance's content list.
    #[serde(default)]
    pub name: Option<String>,
}

/// The input of `resolveInstallPlan`: which content to install, and into what. One shape for a
/// single mod and a whole modpack, for installing and updating alike.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveInstallPlanParamsDto {
    pub content_id: String,
    /// Absent — the provider picks among the versions compatible with the target, by the
    /// release channel: the newest release, or with `installedVersion` set, the newest version
    /// of its channel (a beta takes betas and releases) that is newer than it — never an older
    /// one; with nothing newer, the installed version itself.
    #[serde(default)]
    pub version: Option<String>,
    /// The platform's id of the version of this content installed in the target now; absent —
    /// none is, or the launcher does not know it.
    #[serde(default)]
    pub installed_version: Option<String>,
    pub target: ResolveTargetDto,
}

/// Where the resolved plan is going to be applied; the plan's `InstallTargetDto` answers it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ResolveTargetDto {
    /// Install into, or update, an existing instance, with the game version and loader it runs
    /// now. For most content they are a requirement: it has to fit what the instance runs. For
    /// the pack the instance was installed from (its `packInfo`) they are only the current
    /// state: the plan of an update names the pack's game in `InstallTargetDto::Existing::game`,
    /// and the launcher moves the instance to it.
    Existing {
        instance_id: String,
        game_version: String,
        /// `fabric`, `neoforge`, …; absent for a vanilla instance.
        #[serde(default)]
        loader: Option<String>,
    },
    /// A new instance, described by the content (a modpack).
    New,
}

/// The input of `resolvePackMetadata`: where to read a pack from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PackSourceDto {
    /// A pack hosted on the provider's platform; absent `version` — the version an update would
    /// take from `installedVersion`, by the same rule as `ResolveInstallPlanParamsDto::version`,
    /// which is how a check for updates asks.
    Content {
        content_id: String,
        #[serde(default)]
        version: Option<String>,
        /// The version of the pack the instance runs now.
        #[serde(default)]
        installed_version: Option<String>,
    },
    /// A pack file to download (a `.mrpack`, a packwiz `pack.toml`).
    Url(String),
    /// A pack file the plugin can read: a plugin-side path under `/cache` or `/instances`.
    File(String),
}

/// The answer of `resolvePackMetadata`: a pack's description, read without resolving its files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMetadataDto {
    pub name: String,
    /// The platform's id of this version — what a check for updates compares. Absent when the
    /// pack was not read from the platform and the provider cannot tell.
    #[serde(default)]
    pub version_id: Option<String>,
    /// The version as the pack's author wrote it (`1.4.2`), for display only.
    #[serde(default)]
    pub version_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub game_version: String,
    pub mod_loader: ModLoaderDto,
    /// Absent — the pack does not pin one.
    #[serde(default)]
    pub loader_version: Option<String>,
    /// How many files installing the pack would bring, when known without resolving them.
    #[serde(default)]
    pub file_count: Option<u64>,
    #[serde(default)]
    pub icon_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_write_a_unit_target_as_its_name() {
        assert_eq!(
            serde_json::to_string(&ResolveTargetDto::New).unwrap(),
            r#""new""#
        );
        assert_eq!(
            serde_json::to_string(&ResolveTargetDto::Existing {
                instance_id: "i".into(),
                game_version: "1.21.1".into(),
                loader: None,
            })
            .unwrap(),
            r#"{"existing":{"instanceId":"i","gameVersion":"1.21.1","loader":null}}"#
        );
    }

    /// A launcher from before `installedVersion` does not send it.
    #[test]
    fn should_read_plan_params_without_an_installed_version() {
        let params: ResolveInstallPlanParamsDto =
            serde_json::from_str(r#"{ "contentId": "AANobbMI", "target": "new" }"#).unwrap();

        assert_eq!(params.installed_version, None);
    }

    #[test]
    fn should_read_a_pack_source_without_a_version() {
        let source: PackSourceDto =
            serde_json::from_str(r#"{ "content": { "contentId": "1KVo5zza" } }"#).unwrap();

        assert_eq!(
            source,
            PackSourceDto::Content {
                content_id: "1KVo5zza".into(),
                version: None,
                installed_version: None,
            }
        );
    }
}
