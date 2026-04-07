use aether_core_plugin_api::v0::{CommandDto, OutputDto};
use extism::host_fn;
use extism_convert::Msgpack;

use crate::{core::LauncherState, shared::execute_async};

use super::super::{
    super::{
        super::plugin_utils,
        mappers::{to_extism_res, OutputDtoExt},
    },
    PluginContext,
};

host_fn!(
pub log(user_data: PluginContext; level: u32, msg: String) -> () {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    log::log!(target: "plugin", plugin_utils::log_level_from_u32(level), "[{}]: {}", id, msg);
    Ok(())
});

host_fn!(
pub get_id(user_data: PluginContext;) -> String {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    Ok(id)
});

host_fn!(
pub run_command(user_data: PluginContext; command: Msgpack<CommandDto>) -> HostResult<OutputDto> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    let command = command.0;

    let command_for_log = command.clone();
    log::debug!("Processing command from plugin: {:?}", command_for_log);

    to_extism_res::<OutputDto>(
        execute_async(async move {
            let id = id.clone();
            let command = command.clone();

            let state = LauncherState::get().await?;

            let host_command = plugin_utils::plugin_command_to_host(&id, &command, &state.location_info)?;
            let mut cmd = host_command.to_tokio_command();

            log::debug!("Running command: {:?}", host_command);
            let output = cmd.output().await;

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        log::error!("Command failed: {:?}, stderr: {:?}", command_for_log, String::from_utf8_lossy(&output.stderr));
                        return Err(crate::ErrorKind::CoreError("Command execution failed".to_string()).as_error());
                    }

                    Ok(OutputDto::from_output(&output))
                },
                Err(err) => {
                    log::debug!("Update command run error {:?}", err);
                    Err(crate::ErrorKind::CoreError(format!("Failed to run command: {:?}", cmd)).as_error())
                }
            }
        })
    )
});
