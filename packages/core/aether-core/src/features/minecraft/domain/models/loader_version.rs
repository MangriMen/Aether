#[derive(Debug, Clone, Default)]
pub enum LoaderVersionPreference {
    Latest,
    #[default]
    Stable,
    Exact(String),
}
