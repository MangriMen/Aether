use serde::{Deserialize, Serialize};

use super::ContentTypeDto;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdDto {
    pub plugin_id: String,
    pub capability_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileDto {
    pub content_path: String,
    pub content_type: ContentTypeDto,
    pub disabled: bool,
    pub file_name: String,
    pub hash: String,
    pub name: Option<String>,
    pub size: u64,
    pub update_provider_id: Option<ProviderIdDto>,
    /// Where the file can be updated from, one entry per provider.
    ///
    /// A list rather than a map keyed by provider: a map key would need a string form of
    /// [`ProviderIdDto`], and a plugin id may itself contain the separator.
    pub update: Option<Vec<ContentFileUpdateInfoDto>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileUpdateInfoDto {
    pub provider_id: ProviderIdDto,
    pub content_id: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v0::ContentTypeDto;

    #[test]
    fn test_content_file_camel_case() {
        let file = ContentFileDto {
            content_path: "mods/content.jar".into(),
            content_type: ContentTypeDto::Mod,
            disabled: false,
            file_name: "mod.jar".into(),
            hash: "123".into(),
            name: None,
            size: 1024,
            update_provider_id: None,
            update: None,
        };

        let json = serde_json::to_string(&file).unwrap();

        assert!(json.contains(r#""contentPath""#));
    }

    #[test]
    fn should_round_trip_update_info_of_a_builtin_provider() {
        // A plugin id may contain the separator used in the string form of a provider id.
        let provider_id = ProviderIdDto {
            plugin_id: "core:modrinth".into(),
            capability_id: "modrinth-content".into(),
        };
        let file = ContentFileDto {
            content_path: "mods/sodium.jar".into(),
            content_type: ContentTypeDto::Mod,
            disabled: false,
            file_name: "sodium.jar".into(),
            hash: "123".into(),
            name: None,
            size: 1024,
            update_provider_id: Some(provider_id.clone()),
            update: Some(vec![ContentFileUpdateInfoDto {
                provider_id: provider_id.clone(),
                content_id: "AANobbMI".into(),
                version: "IZskON6d".into(),
            }]),
        };

        let json = serde_json::to_string(&file).unwrap();
        let back: ContentFileDto = serde_json::from_str(&json).unwrap();

        assert_eq!(back.update_provider_id, Some(provider_id));
        assert_eq!(back.update, file.update);
    }
}
