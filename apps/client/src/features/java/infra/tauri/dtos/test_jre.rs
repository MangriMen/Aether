use std::path::PathBuf;

use aether_core::features::java::app::TestJreVersion;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct TestJreVersionDto {
    pub major_version: u32,
    pub path: PathBuf,
}

impl From<TestJreVersionDto> for TestJreVersion {
    fn from(value: TestJreVersionDto) -> Self {
        Self {
            major_version: value.major_version,
            path: value.path,
        }
    }
}
