use crate::{
    features::minecraft::{GameVersion, ModLoader},
    shared::web_url::HttpUrl,
};

use super::{ContentId, ContentVersionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentVersionDependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug, Clone)]
pub struct ContentVersionDependency {
    pub version_id: Option<ContentVersionId>,
    pub content_id: Option<ContentId>,
    pub file_name: Option<String>,
    pub dependency_type: ContentVersionDependencyType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentVersionType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentVersionStatus {
    Listed,
    Archived,
    Draft,
    Unlisted,
    Scheduled,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestedContentVersionStatus {
    Listed,
    Archived,
    Draft,
    Unlisted,
}

/// One file of a version, as the page of the version lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentVersionFile {
    pub file_name: String,
    /// In bytes.
    pub size: u64,
    /// The file the provider names as the main one of the version.
    pub primary: bool,
}

#[derive(Debug, Clone)]
pub struct ContentVersion {
    pub id: ContentVersionId,
    pub content_id: ContentId,
    pub author_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<ContentVersionDependency>,
    pub game_versions: Vec<GameVersion>,
    pub version_type: ContentVersionType,
    pub loaders: Vec<ModLoader>,
    pub featured: bool,
    pub status: ContentVersionStatus,
    pub requested_status: Option<RequestedContentVersionStatus>,
    pub date_published: String,
    pub downloads: u64,
    pub files: Vec<ContentVersionFile>,
    /// The version's page, to link to.
    pub web_url: Option<HttpUrl>,
}
