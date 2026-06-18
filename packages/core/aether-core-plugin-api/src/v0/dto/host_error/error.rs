use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "status", content = "data", rename_all = "camelCase")]
pub enum HostResult<T> {
    Ok(T),
    Err(HostError),
}

pub type Result<T> = HostResult<T>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum HostError {
    AuthError(super::AuthErrorDto),
    EventError(super::EventErrorDto),
    FileWatcherError(super::FileWatcherErrorDto),
    InstanceError(super::InstanceErrorDto),
    JavaError(super::JavaErrorDto),
    MinecraftError(super::MinecraftErrorDto),
    PluginError(super::PluginErrorDto),
    ProcessError(super::ProcessErrorDto),
    SettingsError(super::SettingsErrorDto),
    RequestError(super::RequestErrorDto),
    CoreError(String),
}
