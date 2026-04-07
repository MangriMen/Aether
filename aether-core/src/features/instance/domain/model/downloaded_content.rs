use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::features::instance::ContentFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedContent {
    pub metadata: ContentFile,
    pub temp_path: PathBuf,
}
