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

    #[test]
    fn serializes_every_variant_as_lowercase_string() {
        let cases = [
            (ModLoaderDto::Vanilla, "\"vanilla\""),
            (ModLoaderDto::Forge, "\"forge\""),
            (ModLoaderDto::Fabric, "\"fabric\""),
            (ModLoaderDto::Quilt, "\"quilt\""),
            (ModLoaderDto::NeoForge, "\"neoforge\""),
        ];
        for (variant, expected) in cases {
            assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
            let back: ModLoaderDto = serde_json::from_str(expected).unwrap();
            assert_eq!(serde_json::to_string(&back).unwrap(), expected);
        }
    }
}
