use serde::{Deserialize, Serialize};

/// The input of `getContentsMetadata`: the projects to describe, asked in one go for a whole
/// instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContentsMetadataParamsDto {
    /// The provider's ids of the projects, or their slugs.
    pub content_ids: Vec<String>,
}

/// The input of `getVersionsMetadata`: the versions to describe, asked in one go for a whole
/// instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVersionsMetadataParamsDto {
    /// The provider's ids of the versions, as `ContentFileUpdateInfo::version` records them.
    pub version_ids: Vec<String>,
}

/// The input of `getAuthors`: the authors to name, asked in one go for a whole instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAuthorsParamsDto {
    /// The `authorRef`s the provider gave in `ContentMetadataDto`.
    pub author_refs: Vec<String>,
}

/// One item of the answer of `getContentsMetadata`: what the provider says about a project, to
/// show installed content by.
///
/// The answer is a list of these, for the ids of `GetContentsMetadataParamsDto`. An id the
/// provider does not know is left out of the answer, not an error.
///
/// The host keeps only items that answer an asked id by their `contentId` or by their `slug`
/// (compared without regard to case), and only the first of items with one `contentId`. An item
/// is kept under its `contentId`, so do not give an item the `slug` of one asked project and the
/// `contentId` of another. An item
/// without a `contentId` or a non-blank `name` is dropped; the rest of the answer is kept.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetadataDto {
    /// The provider's own id of the project, which may differ from the one asked by (a slug, for
    /// one).
    pub content_id: String,
    /// The project's short name in URLs. Ids are also asked by it, so the host matches it to an
    /// asked id.
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Absent when the provider cannot tell it without extra requests: the rest is shown alone.
    #[serde(default)]
    pub author: Option<String>,
    /// The provider's own token for whoever authors the project (a team, an organization), when
    /// naming them is a request apart: the host asks `getAuthors` for it later, so the project is
    /// shown even when that request fails. Opaque to the host; used instead of `author` or
    /// besides it. A blank ref is no ref.
    #[serde(default)]
    pub author_ref: Option<String>,
    /// An `https` URL of the icon. The host downloads it itself, so the plugin needs no network
    /// access to it; a URL of another scheme, or naming `localhost` or an address of a private
    /// network, is dropped.
    #[serde(default)]
    pub icon_url: Option<String>,
}

/// One item of the answer of `getVersionsMetadata`: what the provider says about a version, to
/// show installed files by.
///
/// The answer is a list of these, for the ids of `GetVersionsMetadataParamsDto`. An id the
/// provider does not know is left out of the answer, not an error.
///
/// The host keeps only items whose `versionId` was asked, once each, and with a non-blank
/// `versionNumber`; the rest of the answer is kept when one item is dropped.
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

/// One item of the answer of `getAuthors`: who authors projects, by the `authorRef` of
/// `ContentMetadataDto`.
///
/// The answer is a list of these, for the refs of `GetAuthorsParamsDto`. A ref the provider does
/// not know is left out of the answer, not an error.
///
/// The host keeps only items whose `authorRef` was asked, once each, and with a non-blank
/// `name`; the rest of the answer is kept when one item is dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentAuthorDto {
    /// The ref, as asked.
    pub author_ref: String,
    /// The name to show as the author.
    pub name: String,
}
