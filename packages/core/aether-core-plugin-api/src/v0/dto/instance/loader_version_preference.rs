use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreferenceDto {
    Latest,
    #[default]
    Stable,
    #[serde(untagged)]
    Exact(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_version_serde() {
        // Тест Exact версии (untagged)
        let exact: LoaderVersionPreferenceDto = serde_json::from_str(r#""1.20.1""#).unwrap();
        match exact {
            LoaderVersionPreferenceDto::Exact(v) => assert_eq!(v, "1.20.1"),
            _ => panic!("Expected Exact variant"),
        }

        // Тест Stable (именованный вариант)
        let stable: LoaderVersionPreferenceDto = serde_json::from_str(r#""stable""#).unwrap();
        assert!(matches!(stable, LoaderVersionPreferenceDto::Stable));
    }
}
