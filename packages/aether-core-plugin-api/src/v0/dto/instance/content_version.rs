use serde::{Deserialize, Serialize};

use crate::v0::ModLoaderDto;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentVersionDependencyTypeDto {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVersionDependencyDto {
    pub version_id: Option<String>,
    pub content_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: ContentVersionDependencyTypeDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentVersionTypeDto {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentVersionStatusDto {
    Listed,
    Archived,
    Draft,
    Unlisted,
    Scheduled,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestedContentVersionStatusDto {
    Listed,
    Archived,
    Draft,
    Unlisted,
}

/// One file of a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionFileDto {
    pub file_name: String,
    /// In bytes.
    pub size: u64,
    /// The main file of the version.
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// The files of the version; a plugin that does not say leaves the list empty.
    #[serde(default)]
    pub files: Vec<ContentVersionFileDto>,
    /// The version's page, an `http` or `https` URL; empty when it has none. Any other value is
    /// dropped by the host.
    pub web_url: String,
}
