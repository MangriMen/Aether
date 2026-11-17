use aether_core_plugin_api::v0::{JavaDto, JavaVersionParamsDto};

use crate::{
    core::app::AetherContainer,
    features::java::{InstallJava, JavaFeature},
};

use super::super::{super::mappers::to_extism_res, PluginContext, block_on};

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
        .execute(InstallJava::new(version), None)
        .await
        .map_err(Into::into)
        .map(Into::into)
}

// ── Extism host function wrappers ──

plugin_host_fn!(
pub get_java(user_data: PluginContext; params: Wire<JavaVersionParamsDto>) -> HostReply<JavaDto> {
    let container = PluginContext::snapshot(&user_data)?.upgrade_container()?;

    Ok(to_extism_res::<JavaDto>(
        block_on(handle_get_java(params.0.version, &container))
    ))
});

plugin_host_fn!(
pub install_java(user_data: PluginContext; params: Wire<JavaVersionParamsDto>) -> HostReply<JavaDto> {
    let container = PluginContext::snapshot(&user_data)?.upgrade_container()?;

    Ok(to_extism_res::<JavaDto>(
        block_on(handle_install_java(params.0.version, &container))
    ))
});
