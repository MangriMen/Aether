import semver from 'semver';

export const checkIsApiCompatible = (
  launcherApiVersion: string,
  pluginApiVersion: string,
) => semver.satisfies(launcherApiVersion, pluginApiVersion);
