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
    use std::str::FromStr;

    // ── ActionOnInstanceLaunch tests ──

    #[test]
    fn action_default_is_nothing() {
        assert_eq!(
            ActionOnInstanceLaunch::default(),
            ActionOnInstanceLaunch::Nothing
        );
    }

    #[test]
    fn action_as_str_returns_lowercase() {
        assert_eq!(ActionOnInstanceLaunch::Nothing.as_str(), "nothing");
        assert_eq!(ActionOnInstanceLaunch::Hide.as_str(), "hide");
        assert_eq!(ActionOnInstanceLaunch::Close.as_str(), "close");
    }

    #[test]
    fn action_display_matches_as_str() {
        assert_eq!(format!("{}", ActionOnInstanceLaunch::Nothing), "nothing");
        assert_eq!(format!("{}", ActionOnInstanceLaunch::Hide), "hide");
        assert_eq!(format!("{}", ActionOnInstanceLaunch::Close), "close");
    }

    #[test]
    fn action_from_str_valid_cases() {
        assert_eq!(
            ActionOnInstanceLaunch::from_str("nothing").unwrap(),
            ActionOnInstanceLaunch::Nothing
        );
        assert_eq!(
            ActionOnInstanceLaunch::from_str("hide").unwrap(),
            ActionOnInstanceLaunch::Hide
        );
        assert_eq!(
            ActionOnInstanceLaunch::from_str("close").unwrap(),
            ActionOnInstanceLaunch::Close
        );
    }

    #[test]
    fn action_from_str_invalid_returns_error() {
        assert!(ActionOnInstanceLaunch::from_str("invalid").is_err());
    }

    #[test]
    fn action_equality() {
        assert_eq!(ActionOnInstanceLaunch::Hide, ActionOnInstanceLaunch::Hide);
        assert_ne!(
            ActionOnInstanceLaunch::Hide,
            ActionOnInstanceLaunch::Nothing
        );
    }

    #[test]
    fn action_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ActionOnInstanceLaunch::Nothing);
        assert!(set.contains(&ActionOnInstanceLaunch::Nothing));
    }

    // ── WindowEffect tests ──

    #[test]
    fn window_effect_default_is_off() {
        assert_eq!(WindowEffect::default(), WindowEffect::Off);
    }

    #[test]
    fn window_effect_as_str_returns_lowercase() {
        assert_eq!(WindowEffect::Off.as_str(), "off");
        assert_eq!(WindowEffect::MicaLight.as_str(), "mica_light");
        assert_eq!(WindowEffect::MicaDark.as_str(), "mica_dark");
        assert_eq!(WindowEffect::Mica.as_str(), "mica");
        assert_eq!(WindowEffect::Acrylic.as_str(), "acrylic");
    }

    #[test]
    fn window_effect_display_matches_as_str() {
        assert_eq!(format!("{}", WindowEffect::Off), "off");
        assert_eq!(format!("{}", WindowEffect::MicaLight), "mica_light");
        assert_eq!(format!("{}", WindowEffect::MicaDark), "mica_dark");
        assert_eq!(format!("{}", WindowEffect::Mica), "mica");
        assert_eq!(format!("{}", WindowEffect::Acrylic), "acrylic");
    }

    #[test]
    fn window_effect_from_str_valid_cases() {
        assert_eq!(WindowEffect::from_str("off").unwrap(), WindowEffect::Off);
        assert_eq!(
            WindowEffect::from_str("mica_light").unwrap(),
            WindowEffect::MicaLight
        );
        assert_eq!(
            WindowEffect::from_str("mica_dark").unwrap(),
            WindowEffect::MicaDark
        );
        assert_eq!(WindowEffect::from_str("mica").unwrap(), WindowEffect::Mica);
        assert_eq!(
            WindowEffect::from_str("acrylic").unwrap(),
            WindowEffect::Acrylic
        );
    }

    #[test]
    fn window_effect_from_str_invalid_returns_error() {
        let result = WindowEffect::from_str("unknown");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid window effect");
    }

    #[test]
    fn window_effect_roundtrip_all_variants() {
        let variants = [
            WindowEffect::Off,
            WindowEffect::MicaLight,
            WindowEffect::MicaDark,
            WindowEffect::Mica,
            WindowEffect::Acrylic,
        ];
        for variant in variants {
            let serialized = variant.as_str();
            let deserialized = WindowEffect::from_str(serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn window_effect_equality() {
        assert_eq!(WindowEffect::Mica, WindowEffect::Mica);
        assert_ne!(WindowEffect::Mica, WindowEffect::Acrylic);
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
