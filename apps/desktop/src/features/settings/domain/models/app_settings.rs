use std::str::FromStr;

#[derive(Debug, Clone, Copy, Default)]
pub struct AppSettings {
    pub action_on_instance_launch: ActionOnInstanceLaunch,
    pub is_actual_transparent: bool,
    pub transparent: bool,
    pub window_effect: WindowEffect,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub enum ActionOnInstanceLaunch {
    #[default]
    Nothing,
    Hide,
    Close,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::str::FromStr;

    // ── ActionOnInstanceLaunch tests ──

    #[rstest]
    #[case::nothing(ActionOnInstanceLaunch::Nothing, "nothing")]
    #[case::hide(ActionOnInstanceLaunch::Hide, "hide")]
    #[case::close(ActionOnInstanceLaunch::Close, "close")]
    fn action_as_str_returns_lowercase(
        #[case] action: ActionOnInstanceLaunch,
        #[case] expected: &str,
    ) {
        assert_eq!(action.as_str(), expected);
    }

    #[rstest]
    #[case::nothing(ActionOnInstanceLaunch::Nothing, "nothing")]
    #[case::hide(ActionOnInstanceLaunch::Hide, "hide")]
    #[case::close(ActionOnInstanceLaunch::Close, "close")]
    fn action_display_matches_as_str(
        #[case] action: ActionOnInstanceLaunch,
        #[case] expected: &str,
    ) {
        assert_eq!(format!("{action}"), expected);
    }

    #[rstest]
    #[case::nothing("nothing", ActionOnInstanceLaunch::Nothing)]
    #[case::hide("hide", ActionOnInstanceLaunch::Hide)]
    #[case::close("close", ActionOnInstanceLaunch::Close)]
    fn action_from_str_valid_cases(#[case] input: &str, #[case] expected: ActionOnInstanceLaunch) {
        assert_eq!(ActionOnInstanceLaunch::from_str(input).unwrap(), expected);
    }

    #[test]
    fn action_from_str_invalid_returns_error() {
        assert!(ActionOnInstanceLaunch::from_str("invalid").is_err());
    }

    // ── WindowEffect tests ──

    #[rstest]
    #[case::off(WindowEffect::Off, "off")]
    #[case::mica_light(WindowEffect::MicaLight, "mica_light")]
    #[case::mica_dark(WindowEffect::MicaDark, "mica_dark")]
    #[case::mica(WindowEffect::Mica, "mica")]
    #[case::acrylic(WindowEffect::Acrylic, "acrylic")]
    fn window_effect_as_str_returns_lowercase(
        #[case] effect: WindowEffect,
        #[case] expected: &str,
    ) {
        assert_eq!(effect.as_str(), expected);
    }

    #[rstest]
    #[case::off(WindowEffect::Off, "off")]
    #[case::mica_light(WindowEffect::MicaLight, "mica_light")]
    #[case::mica_dark(WindowEffect::MicaDark, "mica_dark")]
    #[case::mica(WindowEffect::Mica, "mica")]
    #[case::acrylic(WindowEffect::Acrylic, "acrylic")]
    fn window_effect_display_matches_as_str(#[case] effect: WindowEffect, #[case] expected: &str) {
        assert_eq!(format!("{effect}"), expected);
    }

    #[rstest]
    #[case::off("off", WindowEffect::Off)]
    #[case::mica_light("mica_light", WindowEffect::MicaLight)]
    #[case::mica_dark("mica_dark", WindowEffect::MicaDark)]
    #[case::mica("mica", WindowEffect::Mica)]
    #[case::acrylic("acrylic", WindowEffect::Acrylic)]
    fn window_effect_from_str_valid_cases(#[case] input: &str, #[case] expected: WindowEffect) {
        assert_eq!(WindowEffect::from_str(input).unwrap(), expected);
    }

    #[test]
    fn window_effect_from_str_invalid_returns_error() {
        let result = WindowEffect::from_str("unknown");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid window effect");
    }

    // ── AppSettings tests ──

    #[test]
    fn app_settings_default_values() {
        let settings = AppSettings::default();
        assert_eq!(
            settings.action_on_instance_launch,
            ActionOnInstanceLaunch::Nothing
        );
        assert!(!settings.transparent);
        assert!(!settings.is_actual_transparent);
        assert_eq!(settings.window_effect, WindowEffect::Off);
    }

    #[test]
    fn app_settings_fields_mutate_independently() {
        let settings = AppSettings {
            transparent: true,
            window_effect: WindowEffect::Acrylic,
            ..Default::default()
        };
        assert!(settings.transparent);
        assert!(!settings.is_actual_transparent);
        assert_eq!(settings.window_effect, WindowEffect::Acrylic);
        assert_eq!(
            settings.action_on_instance_launch,
            ActionOnInstanceLaunch::Nothing
        );
    }
}
