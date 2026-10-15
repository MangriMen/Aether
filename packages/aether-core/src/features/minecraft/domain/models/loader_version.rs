#[derive(Debug, Clone, Default, PartialEq)]
pub enum LoaderVersionPreference {
    Latest,
    #[default]
    Stable,
    Exact(String),
}
