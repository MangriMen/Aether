export enum InstanceInstallStage {
  /// Instance is installed
  Installed = 'installed',
  /// Instance's minecraft game is still installing
  Installing = 'installing',
  /// Instance created for pack, but the pack hasn't been fully installed yet
  PackInstalling = 'pack_installing',
  /// Instance is not installed
  NotInstalled = 'not_installed',
}
