use aether_core_plugin_api::v0::JavaDto;
use extism::host_fn;

use crate::shared::execute_async::infra::execute_async;

use super::super::{super::mappers::to_extism_res, PluginContext};

// ── Testable business logic ──

pub(crate) async fn handle_get_java(version: u32) -> crate::Result<JavaDto> {
    crate::api::java::get(version).await.map(Into::into)
}

pub(crate) async fn handle_install_java(version: u32) -> crate::Result<JavaDto> {
    crate::api::java::install(version).await.map(Into::into)
}

// ── Extism host function wrappers ──

host_fn!(
pub get_java(user_data: PluginContext; version: u32) -> HostResult<JavaDto> {
    to_extism_res::<JavaDto>(
        execute_async(handle_get_java(version))
    )
});

host_fn!(
pub install_java(user_data: PluginContext; version: u32) -> HostResult<JavaDto> {
    to_extism_res::<JavaDto>(
        execute_async(handle_install_java(version))
    )
});
