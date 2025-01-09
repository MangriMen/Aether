import {
  LoaderManifest,
  ModLoader,
  VersionManifest,
  VersionType,
} from '@/entities/minecraft';

export const getGameVersions = (
  versionManifest: VersionManifest | undefined,
  loader: ModLoader,
  shouldIncludeSnapshots: boolean,
  loaderVersions: {
    fabricVersions: LoaderManifest;
    forgeVersions: LoaderManifest;
    quiltVersions: LoaderManifest;
  },
) => {
  const versions = versionManifest?.versions;

  if (loader === ModLoader.Vanilla) {
    return versions ?? [];
  }

  return (
    versions?.filter((version) => {
      const defaultValue =
        version.type === VersionType.Release || shouldIncludeSnapshots;

      switch (loader) {
        case ModLoader.Fabric:
          return (
            defaultValue &&
            loaderVersions.fabricVersions?.gameVersions.some(
              (x) => version.id === x.id,
            )
          );
        case ModLoader.Forge:
          return (
            defaultValue &&
            loaderVersions.forgeVersions?.gameVersions.some(
              (x) => version.id === x.id,
            )
          );
        case ModLoader.Quilt:
          return (
            defaultValue &&
            loaderVersions.quiltVersions?.gameVersions.some(
              (x) => version.id === x.id,
            )
          );
      }
    }) ?? []
  );
};
