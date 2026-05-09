use aether_core::features::instance::InstanceInstallStage;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum InstanceInstallStageDto {
    /// Instance is installed
    Installed,
    /// Instance's minecraft game is still installing
    Installing,
    /// Instance created for pack, but the pack hasn't been fully installed yet
    PackInstalling,
    /// Instance is not installed
    NotInstalled,
}

impl From<InstanceInstallStageDto> for InstanceInstallStage {
    fn from(value: InstanceInstallStageDto) -> Self {
        match value {
            InstanceInstallStageDto::Installed => Self::Installed,
            InstanceInstallStageDto::Installing => Self::Installing,
            InstanceInstallStageDto::PackInstalling => Self::PackInstalling,
            InstanceInstallStageDto::NotInstalled => Self::NotInstalled,
        }
    }
}

impl From<InstanceInstallStage> for InstanceInstallStageDto {
    fn from(value: InstanceInstallStage) -> Self {
        match value {
            InstanceInstallStage::Installed => Self::Installed,
            InstanceInstallStage::Installing => Self::Installing,
            InstanceInstallStage::PackInstalling => Self::PackInstalling,
            InstanceInstallStage::NotInstalled => Self::NotInstalled,
        }
    }
}
