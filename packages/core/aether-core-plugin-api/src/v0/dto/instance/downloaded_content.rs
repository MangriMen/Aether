use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::v0::ContentFileDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedContentDto {
    pub metadata: ContentFileDto,
    pub temp_path: PathBuf,
}
