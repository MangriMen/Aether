use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::features::settings::{FsAppSettingsStorage, TauriWindowManager};

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub action_on_instance_launch: ActionOnInstanceLaunch,
    pub transparent: bool,
    pub window_effect: WindowEffect,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionOnInstanceLaunch {
    #[default]
    Nothing,
    Hide,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
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
