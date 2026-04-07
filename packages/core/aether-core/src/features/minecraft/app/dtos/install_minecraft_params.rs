use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::features::minecraft::{LoaderVersionPreference, ModLoader};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallMinecraftParams {
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub install_dir: PathBuf,
    pub java_path: Option<String>,
}
