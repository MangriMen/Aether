use std::path::PathBuf;

use crate::features::instance::ContentFile;

#[derive(Debug, Clone)]
pub struct DownloadedContent {
    pub metadata: ContentFile,
    pub temp_path: PathBuf,
}
