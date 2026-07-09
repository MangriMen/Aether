use aether_core_plugin_api::v0::{CommandDto, OutputDto};
use extism::host_fn;
use extism_convert::Msgpack;

use crate::{
    core::app::AetherContainer, features::settings::SettingsFeature,
    shared::execute_async::infra::execute_async,
};

use super::super::{
    super::{
        super::plugin_utils,
        mappers::{OutputDtoExt, to_extism_res},
    },
    PluginContext,
};

// ── Testable business logic ──

/// Handle `log` — forward a log message from the plugin.
pub(crate) fn handle_log(plugin_id: &str, level: u32, msg: &str) {
    log::log!(
        target: "plugin",
        plugin_utils::log_level_from_u32(level),
        "[{plugin_id}]: {msg}"
    );
}

/// Handle `run_command` — execute a shell command on behalf of a plugin.
pub(crate) async fn handle_run_command(
    plugin_id: &str,
    command: CommandDto,
    container: &AetherContainer,
) -> crate::Result<OutputDto> {
    let command_for_log = command.clone();
    log::debug!("Processing command from plugin: {command_for_log:?}");

    let host_command =
        plugin_utils::plugin_command_to_host(plugin_id, &command, &container.location_info())?;
    let mut cmd = host_command.to_tokio_command();

    log::debug!("Running command: {host_command:?}");
    let output = cmd.output().await.map_err(|_err| {
        crate::ErrorKind::CoreError(format!("Failed to run command: {cmd:?}")).as_error()
    })?;

    if !output.status.success() {
        log::error!(
            "Command failed: {:?}, stderr: {:?}",
            command_for_log,
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(crate::ErrorKind::CoreError("Command execution failed".to_string()).as_error());
    }

    Ok(OutputDto::from_output(&output))
}

// ── Extism host function wrappers ──

host_fn!(
pub log(user_data: PluginContext; level: u32, msg: String) -> () {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();

    handle_log(&id, level, &msg);
    Ok(())
});

host_fn!(
pub get_id(user_data: PluginContext;) -> String {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();

    Ok(id)
});

host_fn!(
pub run_command(user_data: PluginContext; command: Msgpack<CommandDto>) -> HostResult<OutputDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let container = ctx.container.clone();
    drop(ctx);

    to_extism_res::<OutputDto>(
        execute_async(handle_run_command(&id, command.0, &container))
    )
});
