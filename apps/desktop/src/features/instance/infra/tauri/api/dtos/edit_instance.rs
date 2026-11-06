use aether_core::features::{
    instance::{EditInstance, InstanceName},
    java::{JavaApplicationError, JavaDomainError, JavaPath},
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::infra::{
    dtos::WindowSettingsDto,
    dtos::{EditHooksDto, MemorySettingsDto},
};

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditInstanceDto {
    #[specta(optional)]
    pub name: Option<String>,

    // Java Path
    #[specta(optional)]
    pub override_java_path: Option<bool>,
    #[specta(optional)]
    pub java_path: Option<String>,

    // Launch Args
    #[specta(optional)]
    pub override_launch_args: Option<bool>,
    #[specta(optional)]
    pub launch_args: Option<Vec<String>>,

    // Env Vars
    #[specta(optional)]
    pub override_env_vars: Option<bool>,
    #[specta(optional)]
    pub env_vars: Option<Vec<(String, String)>>,

    // Memory
    #[specta(optional)]
    pub override_memory: Option<bool>,
    #[specta(optional)]
    pub memory: Option<MemorySettingsDto>,

    // Window
    #[specta(optional)]
    pub override_window_settings: Option<bool>,
    #[specta(optional)]
    pub window: Option<WindowSettingsDto>,

    // Hooks
    #[specta(optional)]
    pub override_hooks: Option<bool>,
    #[specta(optional)]
    pub hooks: Option<EditHooksDto>,
}

/// A path to use, or `None` to clear it when the text is empty.
fn parse_java_path(path: String) -> Result<Option<JavaPath>, aether_core::Error> {
    if path.is_empty() {
        return Ok(None);
    }
    let path = JavaPath::try_from(path)
        .map_err(|err| JavaApplicationError::Domain(JavaDomainError::from(err)))?;
    Ok(Some(path))
}

impl TryFrom<EditInstanceDto> for EditInstance {
    type Error = aether_core::Error;

    fn try_from(value: EditInstanceDto) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name.map(InstanceName::try_from).transpose()?,

            override_java_path: value.override_java_path,
            java_path: value.java_path.map(parse_java_path).transpose()?,

            override_launch_args: value.override_launch_args,
            launch_args: value.launch_args,

            override_env_vars: value.override_env_vars,
            env_vars: value.env_vars,

            override_memory: value.override_memory,
            memory: value.memory.map(TryInto::try_into).transpose()?,

            override_window_settings: value.override_window_settings,
            window: value.window.map(Into::into),

            override_hooks: value.override_hooks,
            hooks: value.hooks.map(Into::into),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_clear_the_java_path_when_the_text_is_empty() {
        assert_eq!(parse_java_path(String::new()).unwrap(), None);
    }

    #[test]
    fn should_refuse_a_relative_java_path() {
        assert!(parse_java_path("java17".to_string()).is_err());
    }

    #[test]
    fn should_accept_an_absolute_java_path() {
        let path = std::env::temp_dir().join("java");

        let parsed = parse_java_path(path.to_string_lossy().into_owned()).unwrap();

        assert_eq!(parsed.unwrap().as_path(), path);
    }
}
