use serde::{Deserialize, Serialize};

/// One item of the answer of `getContentsMetadata`: what the provider says about a project, to
/// show installed content by.
///
/// The input is the list of project ids to describe, asked in one go for a whole instance. An id
/// the provider does not know is left out of the answer, not an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetadataDto {
    /// The provider's own id of the project, which may differ from the one asked by (a slug, for
    /// one).
    pub content_id: String,
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Absent when the provider cannot tell it without extra requests: the rest is shown alone.
    #[serde(default)]
    pub author: Option<String>,
    /// An `http(s)` URL of the icon. The host downloads it itself, so the plugin needs no network
    /// access to it; a URL of another scheme is dropped.
    #[serde(default)]
    pub icon_url: Option<String>,
}

/// One item of the answer of `getVersionsMetadata`: what the provider says about a version, to
/// show installed files by.
///
/// The input is the list of the provider's version ids, as `ContentFileUpdateInfo::version`
/// records them. An id the provider does not know is left out of the answer, not an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionMetadataDto {
    /// The provider's own id of the version, as asked.
    pub version_id: String,
    /// The version as its authors number it (`0.6.13+mc1.21.4`).
    pub version_number: String,
    /// The version's title, when the provider has one apart from its number.
    #[serde(default)]
    pub name: Option<String>,
}
