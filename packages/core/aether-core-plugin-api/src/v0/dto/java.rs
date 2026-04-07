use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaDto {
    pub major_version: u32,
    pub version: String,
    pub architecture: String,
    pub path: String,
}
