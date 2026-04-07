use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

use super::ContentTypeDto;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdDto {
    pub plugin_id: String,
    pub capability_id: String,
}

impl std::fmt::Display for ProviderIdDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.plugin_id, self.capability_id)
    }
}

impl FromStr for ProviderIdDto {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid ProviderId format. Expected plugin:capability".to_string());
        }
        Ok(Self {
            plugin_id: parts[0].to_string(),
            capability_id: parts[1].to_string(),
        })
    }
}

#[serde_as]
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
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub update: Option<HashMap<ProviderIdDto, ContentFileUpdateInfoDto>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileUpdateInfoDto {
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
}
