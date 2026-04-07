use aether_core_plugin_api::v0::{ContentFileDto, NewInstanceDto};
use dashmap::DashMap;
use extism::{convert::Msgpack, host_fn};
use path_slash::PathBufExt;

use crate::{core::LauncherState, shared::execute_async};

use super::super::{super::mappers::to_extism_res, PluginContext};

host_fn!(
pub instance_get_dir(user_data: PluginContext; id: String) -> HostResult<String> {
    to_extism_res::<String>(
        execute_async(async move {
            let state =  LauncherState::get().await?;
            let dir = crate::api::instance::get_dir(&id).await?;
            let relative_path = dir.strip_prefix(state.location_info.config_dir())
                .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
                .to_path_buf();

            Ok(format!("/{}",relative_path.to_slash_lossy()))
        })
    )
});

host_fn!(
pub instance_plugin_get_dir(user_data: PluginContext; instance_id: String) -> HostResult<String> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    to_extism_res::<String>(
        execute_async(async move {
            let state = LauncherState::get().await?;

            let dir = state.location_info.instance_plugin_dir(&instance_id, &id);

            let dir = dir.strip_prefix(state.location_info.config_dir())
                .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
                .to_path_buf();

            Ok(format!("/{}",dir.to_slash_lossy()))
        })
    )
});

host_fn!(
    pub instance_create(
        user_data: PluginContext;
        new_instance_dto: Msgpack<NewInstanceDto>
    ) -> HostResult<String> {
        to_extism_res::<String>(
            execute_async(async move {
                crate::api::instance::create(new_instance_dto.0.into()).await
            })
        )
    }
);

host_fn!(
pub list_content(user_data: PluginContext; id: String) -> HostResult<DashMap<String, ContentFileDto>> {
    to_extism_res::<DashMap<String, ContentFileDto>>(
        execute_async(async move {
            crate::api::instance::list_content(id).await
                .map(|map| map.into_iter()
                    .map(|(key, content)| (key, content.into()))
                    .collect()
                )
        })
    )
});

host_fn!(
pub enable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> HostResult<()> {
    to_extism_res::<()>(
        execute_async(async move {
            crate::api::instance::enable_contents(instance_id, content_paths.0).await
        })
    )
});

host_fn!(
pub disable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> HostResult<()> {
    to_extism_res::<()>(
        execute_async(async move {
            crate::api::instance::disable_contents(instance_id, content_paths.0).await
        })
    )
});
