use crate::features::minecraft::ModLoader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentVersionDependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug, Clone)]
pub struct ContentVersionDependency {
    pub version_id: Option<String>,
    pub content_id: Option<String>,
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
    pub id: String,
    pub content_id: String,
    pub author_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<ContentVersionDependency>,
    pub game_versions: Vec<String>,
    pub version_type: ContentVersionType,
    pub loaders: Vec<ModLoader>,
    pub featured: bool,
    pub status: ContentVersionStatus,
    pub requested_status: Option<RequestedContentVersionStatus>,
    pub date_published: String,
    pub downloads: i64,
    pub web_url: String,
}
