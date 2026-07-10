use aether_core_plugin_api::v0::{ContentFileDto, NewInstanceDto};
use dashmap::DashMap;
use extism::{convert::Msgpack, host_fn};
use path_slash::PathBufExt;

use crate::{
    core::app::AetherContainer,
    features::instance::{
        ChangeContentState, ContentManagementPort, ContentStateAction, InstanceCrudPort,
    },
    shared::execute_async::infra::execute_async,
};

use super::super::{super::mappers::to_extism_res, PluginContext};

// ── Testable business logic ──

pub(crate) async fn handle_instance_get_dir(
    id: &str,
    location_info: &crate::features::settings::LocationInfo,
) -> crate::Result<String> {
    let dir = location_info.instance_dir(id);
    let relative_path = dir
        .strip_prefix(location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

pub(crate) async fn handle_instance_plugin_get_dir(
    plugin_id: &str,
    instance_id: &str,
    location_info: &crate::features::settings::LocationInfo,
) -> crate::Result<String> {
    let dir = location_info.instance_plugin_dir(instance_id, plugin_id);
    let relative_path = dir
        .strip_prefix(location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

pub(crate) async fn handle_instance_create(
    dto: NewInstanceDto,
    container: &AetherContainer,
) -> crate::Result<String> {
    container
        .create_instance_use_case()
        .execute(dto.into())
        .await
        .map_err(Into::into)
}

pub(crate) async fn handle_list_content(
    id: String,
    container: &AetherContainer,
) -> crate::Result<DashMap<String, ContentFileDto>> {
    container
        .list_content_use_case()
        .execute(id)
        .await
        .map_err(Into::into)
        .map(|map| {
            map.into_iter()
                .map(|(key, content)| (key, content.into()))
                .collect()
        })
}

pub(crate) async fn handle_enable_contents(
    instance_id: String,
    content_paths: Vec<String>,
    container: &AetherContainer,
) -> crate::Result<()> {
    container
        .change_content_state_use_case()
        .execute(ChangeContentState::multiple(
            instance_id,
            content_paths,
            ContentStateAction::Enable,
        ))
        .await
        .map_err(Into::into)
}

pub(crate) async fn handle_disable_contents(
    instance_id: String,
    content_paths: Vec<String>,
    container: &AetherContainer,
) -> crate::Result<()> {
    container
        .change_content_state_use_case()
        .execute(ChangeContentState::multiple(
            instance_id,
            content_paths,
            ContentStateAction::Disable,
        ))
        .await
        .map_err(Into::into)
}

// ── Extism host function wrappers ──

host_fn!(
pub instance_get_dir(user_data: PluginContext; id: String) -> MsgpackResult<String> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    let location_info = container.location_info();
    drop(ctx);

    to_extism_res::<String>(
        execute_async(handle_instance_get_dir(&id, &location_info))
    )
});

host_fn!(
pub instance_plugin_get_dir(user_data: PluginContext; instance_id: String) -> MsgpackResult<String> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let plugin_id = ctx.id.clone();
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    let location_info = container.location_info();
    drop(ctx);

    to_extism_res::<String>(
        execute_async(handle_instance_plugin_get_dir(&plugin_id, &instance_id, &location_info))
    )
});

host_fn!(
    pub instance_create(
        user_data: PluginContext;
        new_instance_dto: Msgpack<NewInstanceDto>
    ) -> MsgpackResult<String> {
        let context = user_data.get()?;
        let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
        let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
        drop(ctx);

        to_extism_res::<String>(
            execute_async(handle_instance_create(new_instance_dto.0, &container))
        )
    }
);

host_fn!(
pub list_content(user_data: PluginContext; id: String) -> MsgpackResult<DashMap<String, ContentFileDto>> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<DashMap<String, ContentFileDto>>(
        execute_async(handle_list_content(id, &container))
    )
});

host_fn!(
pub enable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<()>(
        execute_async(handle_enable_contents(instance_id, content_paths.0, &container))
    )
});

host_fn!(
pub disable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<()>(
        execute_async(handle_disable_contents(instance_id, content_paths.0, &container))
    )
});
