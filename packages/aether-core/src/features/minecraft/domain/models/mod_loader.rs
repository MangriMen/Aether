/// Supported mod loaders (plus `Vanilla` for no loader).
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ModLoader {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl ModLoader {
    /// Single source of truth for loader naming: `(name, meta name)`.
    const fn names(self) -> (&'static str, &'static str) {
        match self {
            Self::Vanilla => ("vanilla", "vanilla"),
            Self::Forge => ("forge", "forge"),
            Self::Fabric => ("fabric", "fabric"),
            Self::Quilt => ("quilt", "quilt"),
            Self::NeoForge => ("neoforge", "neo"),
        }
    }

    /// Canonical loader name.
    pub fn as_str(&self) -> &'static str {
        self.names().0
    }

    /// Loader name as used in version metadata (`neo` for `NeoForge`).
    pub fn as_meta_str(&self) -> &'static str {
        self.names().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn should_return_both_strings_for_every_variant() {
        let expected = [
            (ModLoader::Vanilla, "vanilla", "vanilla"),
            (ModLoader::Forge, "forge", "forge"),
            (ModLoader::Fabric, "fabric", "fabric"),
            (ModLoader::Quilt, "quilt", "quilt"),
            (ModLoader::NeoForge, "neoforge", "neo"),
        ];
        for (loader, name, meta) in expected {
            assert_eq!(loader.as_str(), name);
            assert_eq!(loader.as_meta_str(), meta);
        }
    }
}
