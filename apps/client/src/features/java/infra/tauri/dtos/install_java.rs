use aether_core::features::java::app::InstallJava;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct InstallJavaDto {
    pub version: u32,
    pub force: bool,
}

impl From<InstallJavaDto> for InstallJava {
    fn from(value: InstallJavaDto) -> Self {
        Self {
            version: value.version,
            force: value.force,
        }
    }
}
