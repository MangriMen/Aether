use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

pub const MODRINTH_API_URL: &str = "https://api.modrinth.com/v2";
pub const MODRINTH_WEB_URL: &str = "https://modrinth.com";

lazy_static::lazy_static! {
    pub static ref DEFAULT_HEADERS: reqwest::header::HeaderMap = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "User-Agent",
            "MangriMen/aether".to_string().parse().unwrap(),
        );
        headers
    };
}

// --- Request Params ---

#[derive(Serialize, Debug)]
pub struct ProjectSearchParams {
    pub index: SearchIndex,
    pub offset: i64,
    pub limit: i64,

    #[serde(serialize_with = "serialize_modrinth_facets")]
    pub facets: Vec<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SearchIndex {
    Relevance,
    // Downloads,
    // Follows,
    // Newest,
    // Updated,
}

// pub enum Facet {
//     ProjectType(String),
//     Categories(String),
//     Versions(String),
//     License(String),
// }

// impl Facet {
//     pub fn to_string(&self) -> String {
//         match self {
//             Self::ProjectType(v) => format!("project_type:{}", v),
//             Self::Categories(v) => format!("categories:{}", v),
//             Self::Versions(v) => format!("versions:{}", v),
//             Self::License(v) => format!("license:{}", v),
//         }
//     }
// }

fn serialize_modrinth_facets<S>(facets: &[Vec<String>], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if facets.is_empty() {
        return s.serialize_none();
    }
    let json = serde_json::to_string(facets).map_err(serde::ser::Error::custom)?;
    s.serialize_str(&json)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectVersionsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loaders: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_versions: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(
        default = "default_include_changelog",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_changelog: Option<bool>,
}

fn default_include_changelog() -> Option<bool> {
    Some(true)
}

impl Default for ProjectVersionsRequest {
    fn default() -> Self {
        Self {
            loaders: None,
            game_versions: None,
            featured: None,
            include_changelog: Some(true),
        }
    }
}

impl ProjectVersionsRequest {
    pub fn without_changelog() -> Self {
        Self {
            loaders: None,
            game_versions: None,
            featured: None,
            include_changelog: Some(false),
        }
    }
}

// --- Response Objects ---

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSearchResponse {
    pub hits: Vec<Hit>,
    pub offset: i64,
    pub limit: i64,
    pub total_hits: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hit {
    pub project_id: String,
    pub project_type: String,
    pub slug: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: i64,
    pub follows: i64,
    pub icon_url: String,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectResponse {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub body: String,
    pub status: String,
    pub requested_status: Option<serde_json::Value>,
    pub additional_categories: Vec<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Vec<Option<serde_json::Value>>,
    pub organization: Option<serde_json::Value>,
    pub project_type: String,
    pub downloads: i64,
    pub icon_url: Option<String>,
    pub color: Option<i64>,
    pub thread_id: String,
    pub monetization_status: String,
    pub team: String,
    pub body_url: Option<String>,
    pub moderator_message: Option<serde_json::Value>,
    pub published: String,
    pub updated: String,
    pub approved: Option<String>,
    pub queued: Option<String>,
    pub followers: i64,
    pub license: License,
    pub versions: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub gallery: Vec<Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectVersionResponse {
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<ModrinthDependencyResponse>,
    pub game_versions: Vec<String>,
    pub version_type: ModrinthVersionType,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub status: ModrinthVersionStatus,
    pub requested_status: Option<ModrinthRequestedStatus>,
    pub date_published: String,
    pub downloads: i64,
    pub changelog_url: Option<String>,
    pub files: Vec<File>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthDependencyResponse {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: ModrinthDependencyType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModrinthVersionType {
    Release,
    Beta,
    Alpha,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModrinthVersionStatus {
    Listed,
    Archived,
    Draft,
    Unlisted,
    Scheduled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModrinthRequestedStatus {
    Listed,
    Archived,
    Draft,
    Unlisted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModrinthDependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hashes {
    pub sha1: String,
    pub sha512: String,
}

// --- Modpack Index (mrpack) ---

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<ModrinthIndexFile>,
    pub dependencies: ModrinthDependencies,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthIndexFile {
    pub path: String,
    pub hashes: Hashes,
    pub env: Option<ModrinthEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthDependencies {
    pub minecraft: String,
    #[serde(flatten)]
    pub loaders: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthEnv {
    pub client: String,
    pub server: String,
}
