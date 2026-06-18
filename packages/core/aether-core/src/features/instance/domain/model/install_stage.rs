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
