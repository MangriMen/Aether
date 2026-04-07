use aether_core_plugin_api::v0::JavaDto;
use extism::host_fn;

use crate::shared::execute_async;

use super::super::{super::mappers::to_extism_res, PluginContext};

host_fn!(
pub get_java(user_data: PluginContext; version: u32) -> HostResult<JavaDto> {
    to_extism_res::<JavaDto>(
        execute_async(async move {
            crate::api::java::get(version).await.map(|x| x.into())
        })
    )
});

host_fn!(
pub install_java(user_data: PluginContext; version: u32) -> HostResult<JavaDto> {
    to_extism_res::<JavaDto>(
        execute_async(async move {
            crate::api::java::install(version).await.map(|x| x.into())
        })
    )
});
