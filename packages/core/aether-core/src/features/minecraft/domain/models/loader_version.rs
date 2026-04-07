use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreference {
    Latest,
    #[default]
    Stable,
    #[serde(untagged)]
    Exact(String),
}
