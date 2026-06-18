use aether_core_plugin_api::v0::InstanceInstallStageDto;

use crate::features::instance::InstanceInstallStage;

impl From<InstanceInstallStage> for InstanceInstallStageDto {
    fn from(value: InstanceInstallStage) -> Self {
        match value {
            InstanceInstallStage::NotInstalled => Self::NotInstalled,
            InstanceInstallStage::Installing => Self::Installing,
            InstanceInstallStage::PackInstalling => Self::PackInstalling,
            InstanceInstallStage::Installed => Self::Installed,
        }
    }
}
