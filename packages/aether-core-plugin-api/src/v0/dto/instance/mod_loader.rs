use serde::{Deserialize, Serialize};

/// Mod loader an instance can run on. Serialized as a lowercase string
/// (`vanilla`, `forge`, `fabric`, `quilt`, `neoforge`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderDto {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::vanilla(ModLoaderDto::Vanilla, "\"vanilla\"")]
    #[case::forge(ModLoaderDto::Forge, "\"forge\"")]
    #[case::fabric(ModLoaderDto::Fabric, "\"fabric\"")]
    #[case::quilt(ModLoaderDto::Quilt, "\"quilt\"")]
    #[case::neoforge(ModLoaderDto::NeoForge, "\"neoforge\"")]
    fn serializes_a_variant_as_lowercase_string(
        #[case] variant: ModLoaderDto,
        #[case] expected: &str,
    ) {
        assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
        let back: ModLoaderDto = serde_json::from_str(expected).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), expected);
    }
}
