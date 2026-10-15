use std::path::PathBuf;

use crate::features::minecraft::{LoaderVersionPreference, ModLoader};

#[derive(Debug)]
pub struct InstallMinecraftParams {
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub install_dir: PathBuf,
    pub java_path: Option<String>,
}
