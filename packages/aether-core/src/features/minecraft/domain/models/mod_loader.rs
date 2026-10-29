use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ModLoader {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl ModLoader {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Vanilla => "vanilla",
            Self::Forge => "forge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::NeoForge => "neoforge",
        }
    }

    pub fn as_meta_str(&self) -> &'static str {
        match *self {
            Self::Vanilla => "vanilla",
            Self::Forge => "forge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::NeoForge => "neo",
        }
    }
}

/// A mod loader name that is none of [`ModLoader::as_str`] or an alias.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unknown mod loader \"{name}\"")]
pub struct UnknownModLoader {
    pub name: String,
}

impl FromStr for ModLoader {
    type Err = UnknownModLoader;

    /// Parses the names [`ModLoader::as_str`] gives, and `neo` for `NeoForge`; case-insensitive.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "vanilla" => Ok(Self::Vanilla),
            "forge" => Ok(Self::Forge),
            "fabric" => Ok(Self::Fabric),
            "quilt" => Ok(Self::Quilt),
            "neoforge" | "neo" => Ok(Self::NeoForge),
            _ => Err(UnknownModLoader { name: s.to_owned() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ModLoader; 5] = [
        ModLoader::Vanilla,
        ModLoader::Forge,
        ModLoader::Fabric,
        ModLoader::Quilt,
        ModLoader::NeoForge,
    ];

    #[test]
    fn should_parse_back_the_same_loader_when_given_its_str() {
        for loader in ALL {
            assert_eq!(loader.as_str().parse::<ModLoader>(), Ok(loader));
        }
    }

    #[test]
    fn should_parse_neoforge_when_given_the_neo_alias() {
        assert_eq!("neo".parse::<ModLoader>(), Ok(ModLoader::NeoForge));
    }

    #[test]
    fn should_ignore_case_when_parsing_a_loader() {
        assert_eq!("Fabric".parse::<ModLoader>(), Ok(ModLoader::Fabric));
        assert_eq!("NEOFORGE".parse::<ModLoader>(), Ok(ModLoader::NeoForge));
        assert_eq!("NeO".parse::<ModLoader>(), Ok(ModLoader::NeoForge));
    }

    #[test]
    fn should_reject_a_name_that_is_not_a_loader() {
        assert_eq!(
            "liteloader".parse::<ModLoader>(),
            Err(UnknownModLoader {
                name: "liteloader".into()
            })
        );
        assert!("".parse::<ModLoader>().is_err());
        assert!("neo_forge".parse::<ModLoader>().is_err());
    }

    #[test]
    fn should_return_the_name_when_a_loader_is_asked_for_its_str() {
        assert_eq!(ModLoader::Vanilla.as_str(), "vanilla");
        assert_eq!(ModLoader::Forge.as_str(), "forge");
        assert_eq!(ModLoader::Fabric.as_str(), "fabric");
        assert_eq!(ModLoader::Quilt.as_str(), "quilt");
        assert_eq!(ModLoader::NeoForge.as_str(), "neoforge");
    }

    #[test]
    fn should_return_the_meta_name_when_a_loader_is_asked_for_its_meta_str() {
        assert_eq!(ModLoader::Vanilla.as_meta_str(), "vanilla");
        assert_eq!(ModLoader::Forge.as_meta_str(), "forge");
        assert_eq!(ModLoader::Fabric.as_meta_str(), "fabric");
        assert_eq!(ModLoader::Quilt.as_meta_str(), "quilt");
        assert_eq!(ModLoader::NeoForge.as_meta_str(), "neo");
    }
}
