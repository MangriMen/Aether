use aether_core::features::java::Java;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Type)]
#[serde(rename_all = "camelCase")]
pub struct JavaDto {
    major_version: u32,
    version: String,
    architecture: String,
    path: String,
}

impl From<Java> for JavaDto {
    fn from(value: Java) -> Self {
        Self {
            major_version: value.major_version(),
            version: value.version().to_string(),
            architecture: value.architecture().to_string(),
            path: value.path().to_string(),
        }
    }
}
