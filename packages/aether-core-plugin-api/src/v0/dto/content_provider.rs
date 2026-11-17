use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::v0::{
    ContentAuthorDto, ContentCompatibilityCheckParamsDto, ContentCompatibilityResultDto,
    ContentMetadataDto, ContentVersionDto, ContentVersionMetadataDto, InstanceSummaryDto,
};

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginImportInstanceDto {
    pub importer_id: String,
    pub source: ImportSourceDto,
}

/// What an importer reads an instance from: the input of `importInstance` names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ImportSourceDto {
    /// A file to download: an `http` or `https` address, already checked to be a valid one.
    Url { url: String },
    /// A file the plugin can read: a plugin-side path (`/mnt/<drive>/…` for a Windows path).
    File { path: String },
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCheckCompatibilityParamsDto {
    pub instances: Vec<InstanceSummaryDto>,
    pub check_params: ContentCompatibilityCheckParamsDto,
}

/// The input of `getContent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContentParamsDto {
    /// The provider's id of the project.
    pub content_id: String,
}

/// The input of `listVersion`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsParamsDto {
    /// The provider's id of the project.
    pub content_id: String,
}

/// The answer of `listVersion`.
#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsResultDto {
    pub versions: Vec<ContentVersionDto>,
}

/// The input of `getVersion`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVersionParamsDto {
    /// The provider's id of the version.
    pub version_id: String,
}

/// The answer of `getContentsMetadata`.
#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContentsMetadataResultDto {
    pub contents: Vec<ContentMetadataDto>,
}

/// The answer of `getVersionsMetadata`.
#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVersionsMetadataResultDto {
    pub versions: Vec<ContentVersionMetadataDto>,
}

/// The answer of `getAuthors`.
#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAuthorsResultDto {
    pub authors: Vec<ContentAuthorDto>,
}

/// The answer of `checkCompatibility`.
#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCompatibilityResultDto {
    /// By instance id.
    pub results: HashMap<String, ContentCompatibilityResultDto>,
}

/// The input of an updater's export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceParamsDto {
    pub instance_id: String,
}

#[cfg(test)]
mod import_source_tests {
    use super::*;

    #[test]
    fn should_write_an_import_source_as_its_kind_around_its_fields() {
        assert_eq!(
            serde_json::to_string(&ImportSourceDto::Url {
                url: "https://example.com/pack.toml".into()
            })
            .unwrap(),
            r#"{"url":{"url":"https://example.com/pack.toml"}}"#
        );
        assert_eq!(
            serde_json::to_string(&ImportSourceDto::File {
                path: "/mnt/c/pack.zip".into()
            })
            .unwrap(),
            r#"{"file":{"path":"/mnt/c/pack.zip"}}"#
        );
    }
}
