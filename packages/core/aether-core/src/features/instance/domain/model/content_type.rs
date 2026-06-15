use std::path::{Path, PathBuf};

use crate::features::minecraft::ModLoader;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_mod_from_mod_loader_names() {
        let loaders = vec!["fabric".to_string(), "quilt".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::Mod)
        );
    }

    #[test]
    fn should_return_datapack_from_datapack_loader() {
        let loaders = vec!["datapack".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::DataPack)
        );
    }

    #[test]
    fn should_return_shaderpack_from_shader_loaders() {
        let loaders = vec!["iris".to_string(), "optifine".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::ShaderPack)
        );
    }

    #[test]
    fn should_return_resourcepack_from_resource_loaders() {
        let loaders = vec!["vanilla".to_string(), "minecraft".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::ResourcePack)
        );
    }

    #[test]
    fn should_return_none_for_unknown_loaders() {
        let loaders = vec!["unknown".to_string()];
        assert_eq!(ContentType::get_from_loaders(&loaders), None);
    }

    #[test]
    fn should_return_mod_when_multiple_loaders_match_mod_first() {
        let loaders = vec!["fabric".to_string(), "iris".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::Mod)
        );
    }

    #[test]
    fn should_return_none_for_empty_loaders() {
        let loaders: Vec<String> = vec![];
        assert_eq!(ContentType::get_from_loaders(&loaders), None);
    }

    #[test]
    fn should_return_mod_from_mods_folder() {
        let path = Path::new("/some/instance/mods/MyMod.jar");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::Mod)
        );
    }

    #[test]
    fn should_return_datapack_from_datapacks_folder() {
        let path = Path::new("/some/instance/datapacks/MyPack.zip");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::DataPack)
        );
    }

    #[test]
    fn should_return_resourcepack_from_resourcepacks_folder() {
        let path = Path::new("/some/instance/resourcepacks/MyPack.zip");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::ResourcePack)
        );
    }

    #[test]
    fn should_return_shaderpack_from_shaderpacks_folder() {
        let path = Path::new("/some/instance/shaderpacks/MyShader.zip");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::ShaderPack)
        );
    }

    #[test]
    fn should_return_none_for_unknown_folder() {
        let path = Path::new("/some/instance/other/file.txt");
        assert_eq!(ContentType::get_from_parent_folder(path), None);
    }

    #[test]
    fn should_return_none_for_root_path() {
        let path = Path::new("/file.txt");
        assert_eq!(ContentType::get_from_parent_folder(path), None);
    }

    #[test]
    fn should_map_all_variants_via_from_string_and_to_str_roundtrip() {
        for variant in ContentType::iterator() {
            let s = variant.to_str();
            let parsed = ContentType::from_string(s);
            assert_eq!(parsed, Some(variant));
        }
    }

    #[test]
    fn should_map_modpack_via_from_string() {
        assert_eq!(
            ContentType::from_string("modpack"),
            Some(ContentType::Modpack)
        );
    }

    #[test]
    fn should_return_none_for_unknown_string() {
        assert_eq!(ContentType::from_string("unknown"), None);
    }

    #[test]
    fn should_return_loader_names_for_mod() {
        assert_eq!(
            ContentType::Mod.get_loaders(),
            &["fabric", "forge", "quilt", "neoforge"]
        );
    }

    #[test]
    fn should_return_empty_loaders_for_modpack() {
        assert!(ContentType::Modpack.get_loaders().is_empty());
    }

    #[test]
    fn should_return_correct_folder_for_each_variant() {
        assert_eq!(ContentType::Mod.get_folder(), "mods");
        assert_eq!(ContentType::DataPack.get_folder(), "datapacks");
        assert_eq!(ContentType::ResourcePack.get_folder(), "resourcepacks");
        assert_eq!(ContentType::ShaderPack.get_folder(), "shaderpacks");
        assert_eq!(ContentType::Modpack.get_folder(), "");
    }

    #[test]
    fn should_return_correct_name_for_each_variant() {
        assert_eq!(ContentType::Mod.get_name(), "mod");
        assert_eq!(ContentType::DataPack.get_name(), "datapack");
        assert_eq!(ContentType::ResourcePack.get_name(), "resourcepack");
        assert_eq!(ContentType::ShaderPack.get_name(), "shader");
        assert_eq!(ContentType::Modpack.get_name(), "modpack");
    }

    #[test]
    fn should_build_relative_path() {
        assert_eq!(
            ContentType::Mod.get_relative_path("MyMod.jar"),
            PathBuf::from("mods/MyMod.jar")
        );
        assert_eq!(
            ContentType::Modpack.get_relative_path("my_pack.zip"),
            PathBuf::from("my_pack.zip")
        );
    }

    #[test]
    fn should_iterate_over_non_modpack_variants() {
        let variants: Vec<ContentType> = ContentType::iterator().collect();
        assert_eq!(variants.len(), 4);
        assert!(!variants.contains(&ContentType::Modpack));
    }
}
