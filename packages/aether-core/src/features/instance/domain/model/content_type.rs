use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ContentType {
    Modpack,
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
}

/// Order in which loader tags are matched when a project lists tags of several types.
const LOADER_PRIORITY: [ContentType; 4] = [
    ContentType::Mod,
    ContentType::DataPack,
    ContentType::ShaderPack,
    ContentType::ResourcePack,
];

impl ContentType {
    pub fn get_from_loaders(loaders: &[String]) -> Option<Self> {
        LOADER_PRIORITY.into_iter().find(|kind| {
            let known = kind.get_loaders();
            loaders.iter().any(|x| known.contains(&x.as_str()))
        })
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
            ContentType::Mod => &["fabric", "forge", "quilt", "neoforge"],
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
        self.get_name()
    }

    pub fn get_relative_path(&self, filename: &str) -> PathBuf {
        Path::new(self.get_folder()).join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::mod_loaders(&["fabric", "quilt"], Some(ContentType::Mod))]
    #[case::datapack_loader(&["datapack"], Some(ContentType::DataPack))]
    #[case::shader_loaders(&["iris", "optifine"], Some(ContentType::ShaderPack))]
    #[case::resource_loaders(&["vanilla", "minecraft"], Some(ContentType::ResourcePack))]
    #[case::unknown_loaders(&["unknown"], None)]
    #[case::mod_wins_over_shader(&["fabric", "iris"], Some(ContentType::Mod))]
    #[case::empty_loaders(&[], None)]
    fn should_pick_content_type_when_given_loaders(
        #[case] names: &[&str],
        #[case] expected: Option<ContentType>,
    ) {
        let loaders: Vec<String> = names.iter().map(ToString::to_string).collect();
        assert_eq!(ContentType::get_from_loaders(&loaders), expected);
    }

    #[rstest]
    #[case::mods("/some/instance/mods/MyMod.jar", Some(ContentType::Mod))]
    #[case::datapacks("/some/instance/datapacks/MyPack.zip", Some(ContentType::DataPack))]
    #[case::resourcepacks(
        "/some/instance/resourcepacks/MyPack.zip",
        Some(ContentType::ResourcePack)
    )]
    #[case::shaderpacks(
        "/some/instance/shaderpacks/MyShader.zip",
        Some(ContentType::ShaderPack)
    )]
    #[case::unknown_folder("/some/instance/other/file.txt", None)]
    #[case::root("/file.txt", None)]
    fn should_pick_content_type_when_given_a_parent_folder(
        #[case] path: &str,
        #[case] expected: Option<ContentType>,
    ) {
        assert_eq!(
            ContentType::get_from_parent_folder(Path::new(path)),
            expected
        );
    }

    #[test]
    fn should_roundtrip_every_variant_when_converted_through_string() {
        for variant in ContentType::iterator() {
            let s = variant.to_str();
            let parsed = ContentType::from_string(s);
            assert_eq!(parsed, Some(variant));
        }
    }

    #[test]
    fn should_return_loader_names_when_content_is_a_mod() {
        assert_eq!(
            ContentType::Mod.get_loaders(),
            &["fabric", "forge", "quilt", "neoforge"]
        );
    }

    #[test]
    fn should_return_no_loaders_when_content_is_a_modpack() {
        assert!(ContentType::Modpack.get_loaders().is_empty());
    }

    #[test]
    fn should_return_folder_when_given_each_variant() {
        assert_eq!(ContentType::Mod.get_folder(), "mods");
        assert_eq!(ContentType::DataPack.get_folder(), "datapacks");
        assert_eq!(ContentType::ResourcePack.get_folder(), "resourcepacks");
        assert_eq!(ContentType::ShaderPack.get_folder(), "shaderpacks");
        assert_eq!(ContentType::Modpack.get_folder(), "");
    }

    #[test]
    fn should_return_name_when_given_each_variant() {
        assert_eq!(ContentType::Mod.get_name(), "mod");
        assert_eq!(ContentType::DataPack.get_name(), "datapack");
        assert_eq!(ContentType::ResourcePack.get_name(), "resourcepack");
        assert_eq!(ContentType::ShaderPack.get_name(), "shader");
        assert_eq!(ContentType::Modpack.get_name(), "modpack");
    }

    #[test]
    fn should_build_relative_path_when_given_a_file_name() {
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
    fn should_match_mod_loader_enum_when_listing_mod_loaders() {
        use crate::features::minecraft::ModLoader;

        let expected = [
            ModLoader::Fabric.as_str(),
            ModLoader::Forge.as_str(),
            ModLoader::Quilt.as_str(),
            ModLoader::NeoForge.as_str(),
        ];
        let mut actual = ContentType::Mod.get_loaders().to_vec();
        let mut expected = expected.to_vec();
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_pin_string_forms_when_given_each_variant() {
        let all = [
            (ContentType::Modpack, "modpack"),
            (ContentType::Mod, "mod"),
            (ContentType::DataPack, "datapack"),
            (ContentType::ResourcePack, "resourcepack"),
            (ContentType::ShaderPack, "shader"),
        ];
        for (variant, text) in all {
            assert_eq!(variant.to_str(), text);
            assert_eq!(variant.get_name(), text);
            assert_eq!(ContentType::from_string(text), Some(variant));
        }
        assert_eq!(ContentType::from_string(""), None);
        assert_eq!(ContentType::from_string("shaderpack"), None);
        assert_eq!(ContentType::from_string("Mod"), None);
    }

    #[test]
    fn should_pin_loaders_when_given_each_variant() {
        assert_eq!(ContentType::DataPack.get_loaders(), &["datapack"]);
        assert_eq!(
            ContentType::ResourcePack.get_loaders(),
            &["vanilla", "canvas", "minecraft"]
        );
        assert_eq!(ContentType::ShaderPack.get_loaders(), &["iris", "optifine"]);
    }

    #[test]
    fn should_pin_iteration_order() {
        let variants: Vec<ContentType> = ContentType::iterator().collect();
        assert_eq!(
            variants,
            vec![
                ContentType::Mod,
                ContentType::DataPack,
                ContentType::ResourcePack,
                ContentType::ShaderPack,
            ]
        );
    }

    #[test]
    fn should_prefer_by_priority_when_loaders_of_several_types_match() {
        let pick = |names: &[&str]| {
            let loaders: Vec<String> = names.iter().map(ToString::to_string).collect();
            ContentType::get_from_loaders(&loaders)
        };
        assert_eq!(pick(&["minecraft", "iris"]), Some(ContentType::ShaderPack));
        assert_eq!(
            pick(&["minecraft", "datapack"]),
            Some(ContentType::DataPack)
        );
        assert_eq!(pick(&["iris", "datapack"]), Some(ContentType::DataPack));
        assert_eq!(pick(&["canvas", "forge"]), Some(ContentType::Mod));
        assert_eq!(pick(&["neoforge"]), Some(ContentType::Mod));
        assert_eq!(pick(&["canvas"]), Some(ContentType::ResourcePack));
        assert_eq!(pick(&["Fabric"]), None);
    }

    #[test]
    fn should_return_none_when_path_has_no_parent_folder_name() {
        assert_eq!(ContentType::get_from_parent_folder(Path::new("")), None);
        assert_eq!(ContentType::get_from_parent_folder(Path::new("mods")), None);
        assert_eq!(
            ContentType::get_from_parent_folder(Path::new("mods/sub/a.jar")),
            None
        );
        assert_eq!(
            ContentType::get_from_parent_folder(Path::new("mods/a.jar")),
            Some(ContentType::Mod)
        );
    }
}
