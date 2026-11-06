use std::path::PathBuf;

use crate::features::{
    java::JavaPath,
    minecraft::{GameVersion, LoaderVersionPreference, ModLoader},
};

#[derive(Debug)]
pub struct InstallMinecraftParams {
    pub game_version: GameVersion,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub install_dir: PathBuf,
    pub java_path: Option<JavaPath>,
}
