use std::path::PathBuf;

use aether_core::features::java::EditJava;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditJavaDto {
    pub major_version: u32,
    pub path: PathBuf,
}

impl From<EditJavaDto> for EditJava {
    fn from(value: EditJavaDto) -> Self {
        Self {
            major_version: value.major_version,
            path: value.path,
        }
    }
}
