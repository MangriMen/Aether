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
    pub downloads: i64,
    pub web_url: String,
}
