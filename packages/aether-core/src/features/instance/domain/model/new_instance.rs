use crate::features::{
    instance::PackInfo,
    minecraft::{LoaderVersionPreference, ModLoader},
};

/// Everything needed to create an instance.
///
/// Lives in the domain rather than next to `CreateInstanceUseCase` because an
/// [`InstallTarget::New`](super::InstallTarget::New) carries one: a plan that creates its
/// instance is still pure data.
#[derive(Debug, Clone, PartialEq)]
pub struct NewInstance {
    pub name: String,
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub icon_path: Option<String>,
    pub skip_install_instance: Option<bool>,
    pub pack_info: Option<PackInfo>,
}
