use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub action_on_instance_launch: ActionOnInstanceLaunch,
    pub is_actual_transparent: bool,
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

impl ActionOnInstanceLaunch {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionOnInstanceLaunch::Nothing => "nothing",
            ActionOnInstanceLaunch::Hide => "hide",
            ActionOnInstanceLaunch::Close => "close",
        }
    }
}

impl std::fmt::Display for ActionOnInstanceLaunch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ActionOnInstanceLaunch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nothing" => Ok(Self::Nothing),
            "hide" => Ok(Self::Hide),
            "close" => Ok(Self::Close),
            _ => Err("Invalid action".to_string()),
        }
    }
}

impl WindowEffect {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowEffect::Off => "off",
            WindowEffect::MicaLight => "mica_light",
            WindowEffect::MicaDark => "mica_dark",
            WindowEffect::Mica => "mica",
            WindowEffect::Acrylic => "acrylic",
        }
    }
}

impl std::fmt::Display for WindowEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for WindowEffect {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "off" => Ok(WindowEffect::Off),
            "mica_light" => Ok(WindowEffect::MicaLight),
            "mica_dark" => Ok(WindowEffect::MicaDark),
            "mica" => Ok(WindowEffect::Mica),
            "acrylic" => Ok(WindowEffect::Acrylic),
            _ => Err("Invalid window effect".to_string()),
        }
    }
}
