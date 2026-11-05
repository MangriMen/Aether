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
    /// The version's page, to link to.
    pub web_url: Option<HttpUrl>,
}
