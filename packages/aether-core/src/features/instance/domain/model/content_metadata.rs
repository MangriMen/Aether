/// What a provider says about a project of its own, to show an installed file by: the
/// project's name and icon rather than the version name the file was installed with.
///
/// Keyed by the provider and [`Self::content_id`], the provider's own id of the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentMetadata {
    pub content_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub icon_url: Option<String>,
}

/// What a provider says about one version of a project of its own, to show an installed file
/// by: the version number its authors gave it (`0.6.13+mc1.21.4`) rather than the provider's id
/// of the version, which the file records.
///
/// Keyed by the provider and [`Self::version_id`]. A version does not change once published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentVersionMetadata {
    /// The provider's own id of the version, as [`ContentFileUpdateInfo::version`] holds it.
    ///
    /// [`ContentFileUpdateInfo::version`]: super::ContentFileUpdateInfo::version
    pub version_id: String,
    /// The version as its authors number it.
    pub version_number: String,
    /// The version's title (`Sodium 0.6.13`), when the provider has one apart from its number.
    pub name: Option<String>,
}
