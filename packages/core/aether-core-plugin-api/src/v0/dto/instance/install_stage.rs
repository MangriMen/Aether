use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceInstallStageDto {
    Installed,
    Installing,
    PackInstalling,
    NotInstalled,
}
