use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Java {
    major_version: u32,
    version: String,
    architecture: String,
    path: String,
}

impl Java {
    pub fn new(major_version: u32, version: String, architecture: String, path: String) -> Self {
        Self {
            major_version,
            version,
            architecture,
            path,
        }
    }

    pub fn major_version(&self) -> u32 {
        self.major_version
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn architecture(&self) -> &str {
        &self.architecture
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}
