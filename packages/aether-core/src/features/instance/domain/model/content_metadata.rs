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
