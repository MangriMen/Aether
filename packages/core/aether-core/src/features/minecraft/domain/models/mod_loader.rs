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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_loader_as_str() {
        assert_eq!(ModLoader::Vanilla.as_str(), "vanilla");
        assert_eq!(ModLoader::Forge.as_str(), "forge");
        assert_eq!(ModLoader::Fabric.as_str(), "fabric");
        assert_eq!(ModLoader::Quilt.as_str(), "quilt");
        assert_eq!(ModLoader::NeoForge.as_str(), "neoforge");
    }

    #[test]
    fn mod_loader_as_meta_str() {
        assert_eq!(ModLoader::Vanilla.as_meta_str(), "vanilla");
        assert_eq!(ModLoader::Forge.as_meta_str(), "forge");
        assert_eq!(ModLoader::Fabric.as_meta_str(), "fabric");
        assert_eq!(ModLoader::Quilt.as_meta_str(), "quilt");
        assert_eq!(ModLoader::NeoForge.as_meta_str(), "neo");
    }
}
