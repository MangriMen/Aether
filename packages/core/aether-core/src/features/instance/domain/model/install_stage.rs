use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstanceInstallStage {
    /// Instance is installed
    Installed,
    /// Instance's minecraft game is still installing
    Installing,
    /// Instance created for pack, but the pack hasn't been fully installed yet
    PackInstalling,
    /// Instance is not installed
    NotInstalled,
}

impl InstanceInstallStage {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Installed => "installed",
            Self::Installing => "installing",
            Self::PackInstalling => "pack_installing",
            Self::NotInstalled => "not_installed",
        }
    }
}

impl FromStr for InstanceInstallStage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "installed" => Ok(InstanceInstallStage::Installed),
            "installing" => Ok(InstanceInstallStage::Installing),
            "pack_installing" => Ok(InstanceInstallStage::PackInstalling),
            "not_installed" => Ok(InstanceInstallStage::NotInstalled),
            _ => Err(anyhow::Error::msg("Unknown install stage")),
        }
    }
}
