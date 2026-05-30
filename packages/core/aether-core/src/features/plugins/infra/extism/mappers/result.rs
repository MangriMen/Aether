use aether_core_plugin_api::v0::{
    AuthErrorDto, EventErrorDto, FileWatcherErrorDto, HostError, HostResult, InstanceErrorDto,
    JavaErrorDto, MinecraftErrorDto, PluginErrorDto, ProcessErrorDto, RequestErrorDto,
    SettingsErrorDto,
};
use extism_convert::Msgpack;

pub type MsgpackResult<T> = Msgpack<HostResult<T>>;

#[allow(clippy::unnecessary_wraps)]
pub fn to_extism_res<T>(res: crate::Result<T>) -> Result<MsgpackResult<T>, extism::Error> {
    let host_res = match res {
        Ok(v) => HostResult::Ok(v),
        Err(e) => {
            let error = match e.raw.as_ref() {
                crate::ErrorKind::AuthError(_) => {
                    HostError::AuthError(AuthErrorDto::Storage(e.to_string()))
                }
                crate::ErrorKind::EventError(_) => {
                    HostError::EventError(EventErrorDto::SerializeError(e.to_string()))
                }
                crate::ErrorKind::FileWatcherError(_) => {
                    HostError::FileWatcherError(FileWatcherErrorDto::NotifyError(e.to_string()))
                }
                crate::ErrorKind::InstanceError(_) => {
                    HostError::InstanceError(InstanceErrorDto::Storage(e.to_string()))
                }
                crate::ErrorKind::JavaError(_) => {
                    HostError::JavaError(JavaErrorDto::Storage(e.to_string()))
                }
                crate::ErrorKind::MinecraftError(_) => {
                    HostError::MinecraftError(MinecraftErrorDto::StorageFailure {
                        reason: e.to_string(),
                    })
                }
                crate::ErrorKind::PluginError(_) => {
                    HostError::PluginError(PluginErrorDto::Storage(e.to_string()))
                }
                crate::ErrorKind::ProcessError(_) => {
                    HostError::ProcessError(ProcessErrorDto::Io(e.to_string()))
                }
                crate::ErrorKind::SettingsError(_) => {
                    HostError::SettingsError(SettingsErrorDto::Storage(e.to_string()))
                }
                crate::ErrorKind::RequestError(_) => {
                    HostError::RequestError(RequestErrorDto::Acquire(e.to_string()))
                }
                crate::ErrorKind::CoreError(_) => HostError::CoreError(e.to_string()),
            };
            HostResult::Err(error)
        }
    };

    Ok(Msgpack(host_res))
}
