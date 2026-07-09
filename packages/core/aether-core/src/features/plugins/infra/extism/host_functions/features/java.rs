use aether_core_plugin_api::v0::JavaDto;
use extism::host_fn;

use crate::{
    core::app::AetherContainer,
    features::java::{InstallJava, JavaFeature},
    shared::execute_async::infra::execute_async,
};

use super::super::{super::mappers::to_extism_res, PluginContext};

// ── Testable business logic ──

pub(crate) async fn handle_get_java(
    version: u32,
    container: &AetherContainer,
) -> crate::Result<JavaDto> {
    container
        .get_java_use_case()
        .execute(version)
        .await
        .map_err(Into::into)
        .map(Into::into)
}

pub(crate) async fn handle_install_java(
    version: u32,
    container: &AetherContainer,
) -> crate::Result<JavaDto> {
    container
        .install_java_use_case()
        .execute(InstallJava::new(version))
        .await
        .map_err(Into::into)
        .map(Into::into)
}

// ── Extism host function wrappers ──

host_fn!(
pub get_java(user_data: PluginContext; version: u32) -> HostResult<JavaDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.container.clone();
    drop(ctx);

    to_extism_res::<JavaDto>(
        execute_async(handle_get_java(version, &container))
    )
});

host_fn!(
pub install_java(user_data: PluginContext; version: u32) -> HostResult<JavaDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.container.clone();
    drop(ctx);

    to_extism_res::<JavaDto>(
        execute_async(handle_install_java(version, &container))
    )
});
