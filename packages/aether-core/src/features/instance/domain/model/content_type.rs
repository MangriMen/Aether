use std::path::{Path, PathBuf};

/// Kind of content an instance can hold.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ContentType {
    Modpack,
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
}

/// Static facts about one content type.
struct Traits {
    kind: ContentType,
    /// Identifier used in provider APIs and in the frontend.
    id: &'static str,
    /// Directory inside an instance; empty when the content has no directory of its own.
    folder: &'static str,
    /// Provider-side loader tags that mark a project as this type.
    loaders: &'static [&'static str],
}

const TRAITS: [Traits; 5] = [
    Traits {
        kind: ContentType::Modpack,
        id: "modpack",
        folder: "",
        loaders: &[],
    },
    Traits {
        kind: ContentType::Mod,
        id: "mod",
        folder: "mods",
        loaders: &["fabric", "forge", "quilt", "neoforge"],
    },
    Traits {
        kind: ContentType::DataPack,
        id: "datapack",
        folder: "datapacks",
        loaders: &["datapack"],
    },
    Traits {
        kind: ContentType::ResourcePack,
        id: "resourcepack",
        folder: "resourcepacks",
        loaders: &["vanilla", "canvas", "minecraft"],
    },
    Traits {
        kind: ContentType::ShaderPack,
        id: "shader",
        folder: "shaderpacks",
        loaders: &["iris", "optifine"],
    },
];

/// Order in which loader tags are matched when a project lists tags of several types.
const LOADER_PRIORITY: [ContentType; 4] = [
    ContentType::Mod,
    ContentType::DataPack,
    ContentType::ShaderPack,
    ContentType::ResourcePack,
];

impl ContentType {
    fn traits(self) -> &'static Traits {
        TRAITS
            .iter()
            .find(|t| t.kind == self)
            .expect("every content type has a row in TRAITS")
    }

    /// Picks the type for a project from its loader tags; `None` if no tag is recognised.
    pub fn get_from_loaders(loaders: &[String]) -> Option<Self> {
        LOADER_PRIORITY.into_iter().find(|kind| {
            let known = kind.traits().loaders;
            loaders.iter().any(|tag| known.contains(&tag.as_str()))
        })
    }

    /// Guesses the type of a file from the name of the directory it sits in.
    pub fn get_from_parent_folder(path: &Path) -> Option<Self> {
        let dir_name = path.parent()?.file_name()?.to_str()?;
        Self::iterator().find(|kind| kind.traits().folder == dir_name)
    }

    pub fn get_name(&self) -> &'static str {
        self.traits().id
    }

    pub fn get_folder(&self) -> &'static str {
        self.traits().folder
    }

    pub fn get_loaders(&self) -> &'static [&'static str] {
        self.traits().loaders
    }

    /// All types that live in a folder of their own (everything except [`Self::Modpack`]).
    pub fn iterator() -> impl Iterator<Item = ContentType> {
        TRAITS
            .iter()
            .map(|t| t.kind)
            .filter(|kind| *kind != ContentType::Modpack)
    }

    pub fn from_string(s: &str) -> Option<ContentType> {
        TRAITS.iter().find(|t| t.id == s).map(|t| t.kind)
    }

    /// Same value as [`Self::get_name`].
    pub fn to_str(&self) -> &str {
        self.get_name()
    }

    /// Path of `filename` relative to the instance directory.
    pub fn get_relative_path(&self, filename: &str) -> PathBuf {
        Path::new(self.get_folder()).join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_mod_when_given_mod_loader_names() {
        let loaders = vec!["fabric".to_string(), "quilt".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::Mod)
        );
    }

    #[test]
    fn should_return_datapack_when_given_the_datapack_loader() {
        let loaders = vec!["datapack".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::DataPack)
        );
    }

    #[test]
    fn should_return_shaderpack_when_given_shader_loaders() {
        let loaders = vec!["iris".to_string(), "optifine".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::ShaderPack)
        );
    }

    #[test]
    fn should_return_resourcepack_when_given_resource_loaders() {
        let loaders = vec!["vanilla".to_string(), "minecraft".to_string()];
        assert_eq!(
            ContentType::get_from_loaders(&loaders),
            Some(ContentType::ResourcePack)
        );
    }

    #[test]
    fn should_return_none_when_loaders_are_unknown() {
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
    fn should_return_none_when_loaders_are_empty() {
        let loaders: Vec<String> = vec![];
        assert_eq!(ContentType::get_from_loaders(&loaders), None);
    }

    #[test]
    fn should_return_mod_when_folder_is_mods() {
        let path = Path::new("/some/instance/mods/MyMod.jar");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::Mod)
        );
    }

    #[test]
    fn should_return_datapack_when_folder_is_datapacks() {
        let path = Path::new("/some/instance/datapacks/MyPack.zip");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::DataPack)
        );
    }

    #[test]
    fn should_return_resourcepack_when_folder_is_resourcepacks() {
        let path = Path::new("/some/instance/resourcepacks/MyPack.zip");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::ResourcePack)
        );
    }

    #[test]
    fn should_return_shaderpack_when_folder_is_shaderpacks() {
        let path = Path::new("/some/instance/shaderpacks/MyShader.zip");
        assert_eq!(
            ContentType::get_from_parent_folder(path),
            Some(ContentType::ShaderPack)
        );
    }

    #[test]
    fn should_return_none_when_folder_is_unknown() {
        let path = Path::new("/some/instance/other/file.txt");
        assert_eq!(ContentType::get_from_parent_folder(path), None);
    }

    #[test]
    fn should_return_none_when_path_is_the_root() {
        let path = Path::new("/file.txt");
        assert_eq!(ContentType::get_from_parent_folder(path), None);
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
    fn should_return_modpack_when_string_is_modpack() {
        assert_eq!(
            ContentType::from_string("modpack"),
            Some(ContentType::Modpack)
        );
    }

    #[test]
    fn should_return_none_when_string_is_unknown() {
        assert_eq!(ContentType::from_string("unknown"), None);
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
    fn should_skip_modpack_when_iterating_variants() {
        let variants: Vec<ContentType> = ContentType::iterator().collect();
        assert_eq!(variants.len(), 4);
        assert!(!variants.contains(&ContentType::Modpack));
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
