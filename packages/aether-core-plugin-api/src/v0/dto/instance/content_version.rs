use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::ModLoaderDto;

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContentVersionDependencyTypeDto {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionDependencyDto {
    pub version_id: Option<String>,
    pub content_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: ContentVersionDependencyTypeDto,
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContentVersionTypeDto {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContentVersionStatusDto {
    Listed,
    Archived,
    Draft,
    Unlisted,
    Scheduled,
    Unknown,
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RequestedContentVersionStatusDto {
    Listed,
    Archived,
    Draft,
    Unlisted,
}

/// One file of a version.
#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionFileDto {
    pub file_name: String,
    /// In bytes.
    pub size: u64,
    /// The main file of the version.
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionDto {
    pub id: String,
    pub content_id: String,
    pub author_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<ContentVersionDependencyDto>,
    pub game_versions: Vec<String>,
    pub version_type: ContentVersionTypeDto,
    pub loaders: Vec<ModLoaderDto>,
    pub featured: bool,
    pub status: ContentVersionStatusDto,
    pub requested_status: Option<RequestedContentVersionStatusDto>,
    pub date_published: String,
    pub downloads: u64,
    /// The version's page, an `http` or `https` URL; absent when it has none. Any other value is
    /// dropped by the host.
    #[serde(default)]
    pub web_url: Option<String>,
    /// The files of the version. A plugin built before it existed leaves it out and decodes with
    /// an empty list.
    #[serde(default)]
    pub files: Vec<ContentVersionFileDto>,
}
