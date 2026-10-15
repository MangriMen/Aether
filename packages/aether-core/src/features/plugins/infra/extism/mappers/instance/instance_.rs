use aether_core_plugin_api::v0::InstanceDto;

use crate::features::instance::Instance;

impl From<Instance> for InstanceDto {
    fn from(value: Instance) -> Self {
        Self {
            id: value.id().to_owned(),

            name: value.name().to_owned(),

            icon_path: value.icon_path().map(ToString::to_string),

            install_stage: value.install_stage.into(),

            game_version: value.game_version,
            loader: value.loader.into(),
            loader_version: value.loader_version.map(Into::into),

            java_path: value.java_path.into(),
            launch_args: value.launch_args.into(),
            env_vars: value.env_vars.into(),

            memory: value.memory.into(),

            window: value.window.into(),

            created: value.created,
            modified: value.modified,
            last_played: value.last_played,

            time_played: value.time_played,
            recent_time_played: value.recent_time_played,

            hooks: value.hooks.into(),

            pack_info: value.pack_info.map(Into::into),
        }
    }
}
