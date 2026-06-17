use aether_core_plugin_api::v0::{ContentFileDto, NewInstanceDto};
use dashmap::DashMap;
use extism::{convert::Msgpack, host_fn};
use path_slash::PathBufExt;

use crate::{core::LazyLocator, shared::execute_async::infra::execute_async};

use super::super::{super::mappers::to_extism_res, PluginContext};

// ── Testable business logic ──

pub(crate) async fn handle_instance_get_dir(id: &str) -> crate::Result<String> {
    let locator = LazyLocator::get().await?;
    let dir = crate::api::instance::get_dir(id).await?;
    let relative_path = dir
        .strip_prefix(locator.location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

pub(crate) async fn handle_instance_plugin_get_dir(
    plugin_id: &str,
    instance_id: &str,
) -> crate::Result<String> {
    let locator = LazyLocator::get().await?;
    let dir = locator
        .location_info
        .instance_plugin_dir(instance_id, plugin_id);
    let relative_path = dir
        .strip_prefix(locator.location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

pub(crate) async fn handle_instance_create(dto: NewInstanceDto) -> crate::Result<String> {
    crate::api::instance::create(dto.into()).await
}

pub(crate) async fn handle_list_content(
    id: String,
) -> crate::Result<DashMap<String, ContentFileDto>> {
    crate::api::instance::list_content(id).await.map(|map| {
        map.into_iter()
            .map(|(key, content)| (key, content.into()))
            .collect()
    })
}

pub(crate) async fn handle_enable_contents(
    instance_id: String,
    content_paths: Vec<String>,
) -> crate::Result<()> {
    crate::api::instance::enable_contents(instance_id, content_paths).await
}

pub(crate) async fn handle_disable_contents(
    instance_id: String,
    content_paths: Vec<String>,
) -> crate::Result<()> {
    crate::api::instance::disable_contents(instance_id, content_paths).await
}

// ── Extism host function wrappers ──

host_fn!(
pub instance_get_dir(user_data: PluginContext; id: String) -> MsgpackResult<String> {
    to_extism_res::<String>(
        execute_async(handle_instance_get_dir(&id))
    )
});

host_fn!(
pub instance_plugin_get_dir(user_data: PluginContext; instance_id: String) -> MsgpackResult<String> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    to_extism_res::<String>(
        execute_async(handle_instance_plugin_get_dir(&id, &instance_id))
    )
});

host_fn!(
    pub instance_create(
        user_data: PluginContext;
        new_instance_dto: Msgpack<NewInstanceDto>
    ) -> MsgpackResult<String> {
        to_extism_res::<String>(
            execute_async(handle_instance_create(new_instance_dto.0))
        )
    }
);

host_fn!(
pub list_content(user_data: PluginContext; id: String) -> MsgpackResult<DashMap<String, ContentFileDto>> {
    to_extism_res::<DashMap<String, ContentFileDto>>(
        execute_async(handle_list_content(id))
    )
});

host_fn!(
pub enable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    to_extism_res::<()>(
        execute_async(handle_enable_contents(instance_id, content_paths.0))
    )
});

host_fn!(
pub disable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    to_extism_res::<()>(
        execute_async(handle_disable_contents(instance_id, content_paths.0))
    )
});
