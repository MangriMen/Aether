use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::features::minecraft::ModLoader;

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Modpack,
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
}

use std::sync::LazyLock;

static MOD_LOADER_NAMES: LazyLock<[&'static str; 4]> = LazyLock::new(|| {
    [
        ModLoader::Fabric.as_str(),
        ModLoader::Forge.as_str(),
        ModLoader::Quilt.as_str(),
        ModLoader::NeoForge.as_str(),
    ]
});

impl ContentType {
    pub fn get_from_loaders(loaders: &[String]) -> Option<Self> {
        if loaders
            .iter()
            .any(|x| ContentType::Mod.get_loaders().contains(&&**x))
        {
            Some(ContentType::Mod)
        } else if loaders
            .iter()
            .any(|x| ContentType::DataPack.get_loaders().contains(&&**x))
        {
            Some(ContentType::DataPack)
        } else if loaders
            .iter()
            .any(|x| ContentType::ShaderPack.get_loaders().contains(&&**x))
        {
            Some(ContentType::ShaderPack)
        } else if loaders
            .iter()
            .any(|x| ContentType::ResourcePack.get_loaders().contains(&&**x))
        {
            Some(ContentType::ResourcePack)
        } else {
            None
        }
    }

    pub fn get_from_parent_folder(path: &Path) -> Option<Self> {
        let parent_folder = path.parent()?.file_name()?;
        match parent_folder.to_str()? {
            "mods" => Some(ContentType::Mod),
            "datapacks" => Some(ContentType::DataPack),
            "resourcepacks" => Some(ContentType::ResourcePack),
            "shaderpacks" => Some(ContentType::ShaderPack),
            _ => None,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            ContentType::Modpack => "modpack",
            ContentType::Mod => "mod",
            ContentType::DataPack => "datapack",
            ContentType::ResourcePack => "resourcepack",
            ContentType::ShaderPack => "shader",
        }
    }

    pub fn get_folder(&self) -> &'static str {
        match self {
            ContentType::Modpack => "",
            ContentType::Mod => "mods",
            ContentType::DataPack => "datapacks",
            ContentType::ResourcePack => "resourcepacks",
            ContentType::ShaderPack => "shaderpacks",
        }
    }

    pub fn get_loaders(&self) -> &'static [&'static str] {
        match self {
            ContentType::Modpack => &[],
            ContentType::Mod => &*MOD_LOADER_NAMES,
            ContentType::DataPack => &["datapack"],
            ContentType::ResourcePack => &["vanilla", "canvas", "minecraft"],
            ContentType::ShaderPack => &["iris", "optifine"],
        }
    }

    pub fn iterator() -> impl Iterator<Item = ContentType> {
        [
            ContentType::Mod,
            ContentType::DataPack,
            ContentType::ResourcePack,
            ContentType::ShaderPack,
        ]
        .iter()
        .copied()
    }

    pub fn from_string(s: &str) -> Option<ContentType> {
        match s {
            "modpack" => Some(ContentType::Modpack),
            "mod" => Some(ContentType::Mod),
            "datapack" => Some(ContentType::DataPack),
            "resourcepack" => Some(ContentType::ResourcePack),
            "shader" => Some(ContentType::ShaderPack),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            ContentType::Modpack => "modpack",
            ContentType::Mod => "mod",
            ContentType::DataPack => "datapack",
            ContentType::ResourcePack => "resourcepack",
            ContentType::ShaderPack => "shader",
        }
    }

    pub fn get_relative_path(&self, filename: &str) -> PathBuf {
        Path::new(self.get_folder()).join(filename)
    }
}
