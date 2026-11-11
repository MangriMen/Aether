use aether_core_plugin_api::v0::InstanceSummaryDto;

use crate::features::instance::Instance;

impl From<Instance> for InstanceSummaryDto {
    fn from(value: Instance) -> Self {
        Self {
            id: value.id().to_string(),

            name: value.name().to_string(),

            install_stage: value.install_stage.into(),

            game_version: value.game_version.into(),
            loader: value.loader.into(),
            loader_version: value.loader_version.map(Into::into),

            pack_info: value.pack_info.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::{
            instance::{InstanceBuilder, InstanceId, InstanceName},
            minecraft::{GameVersion, ModLoader},
        },
        shared::overridable::domain::Overridable,
    };

    #[test]
    fn should_not_carry_launch_settings_when_an_instance_is_summarised() {
        let mut instance = InstanceBuilder::new(
            InstanceId::generate(),
            InstanceName::parse("pack").unwrap(),
            GameVersion::from_static("1.21.1"),
            ModLoader::Fabric,
        )
        .build();
        instance.env_vars = Overridable::new(vec![("API_KEY".into(), "hunter2".into())], true);
        instance.java_path = Overridable::new("C:/secret/java".into(), true);

        let summary = serde_json::to_string(&InstanceSummaryDto::from(instance)).unwrap();

        for leaked in [
            "API_KEY", "hunter2", "secret", "envVars", "javaPath", "hooks",
        ] {
            assert!(
                !summary.contains(leaked),
                "{leaked} must not reach a plugin"
            );
        }
        assert!(
            summary.contains("1.21.1"),
            "the fields a provider needs stay"
        );
    }
}
