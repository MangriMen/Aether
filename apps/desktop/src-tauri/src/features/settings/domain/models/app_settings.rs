use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::{FsAppSettingsStorage, TauriWindowManager};

#[derive(Clone, Copy, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", rename = "AppSettingsDto")]
pub struct AppSettings {
    pub action_on_instance_launch: ActionOnInstanceLaunch,
    pub transparent: bool,
    pub window_effect: WindowEffect,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Type)]
#[serde(rename_all = "snake_case", rename = "ActionOnInstanceLaunchDto")]
pub enum ActionOnInstanceLaunch {
    #[default]
    Nothing,
    Hide,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Type)]
#[serde(rename_all = "snake_case", rename = "WindowEffectDto")]
pub enum WindowEffect {
    #[default]
    Off,
    MicaLight,
    MicaDark,
    Mica,
    Acrylic,
}

pub type AppSettingsStorageState = Arc<FsAppSettingsStorage>;

pub type WindowManagerState<R> = Arc<TauriWindowManager<R>>;
