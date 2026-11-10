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

/// The game whose shared files (its version, its Java) are downloaded ahead of an install,
/// without an instance to put anything in.
#[derive(Debug)]
pub struct GameDownloadParams {
    pub game_version: GameVersion,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub java_path: Option<JavaPath>,
}
