use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LauncherEvent {
    Loading,
    Process,
    Instance,
    Warning,
    Plugin,
}

impl LauncherEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            LauncherEvent::Loading => "loading",
            LauncherEvent::Process => "process",
            LauncherEvent::Instance => "instance",
            LauncherEvent::Warning => "warning",
            LauncherEvent::Plugin => "plugin",
        }
    }
}
