import type { MappedLoaderManifest, Version } from '@/entities/minecrafts';
import { ModLoader, VersionType } from '@/entities/minecrafts';

export const filterGameVersions = (
  versions: Version[],
  shouldIncludeSnapshots?: boolean,
) =>
  versions.filter(
    (version) => version.type === VersionType.Release || shouldIncludeSnapshots,
  );

export const filterGameVersionsForLoader = (
  loader: ModLoader | undefined,
  versions: Version[],
  loaderVersions: {
    fabric?: MappedLoaderManifest;
    forge?: MappedLoaderManifest;
    quilt?: MappedLoaderManifest;
  },
) => {
  if (!loader || loader === ModLoader.Vanilla) {
    return versions;
  }

  return versions.filter((version) => {
    switch (loader) {
      case ModLoader.Fabric:
        return !!loaderVersions.fabric?.gameVersions?.[version.id];
      case ModLoader.Forge:
        return !!loaderVersions.forge?.gameVersions?.[version.id];
      case ModLoader.Quilt:
        return !!loaderVersions.quilt?.gameVersions?.[version.id];
    }
  });
};

export const getLoaderVersionsForGameVersion = (
  loader: ModLoader | undefined,
  gameVersion: string | undefined,
  loaderVersions: {
    fabric?: MappedLoaderManifest;
    forge?: MappedLoaderManifest;
    quilt?: MappedLoaderManifest;
  },
) => {
  if (!loader || !gameVersion) {
    return [];
  }

  const dummyReplaceString = '${modrinth.gameVersion}';
  switch (loader) {
    case ModLoader.Fabric:
      return (
        loaderVersions.fabric?.gameVersions[dummyReplaceString].loaders ?? []
      );
    case ModLoader.Forge:
      return loaderVersions.forge?.gameVersions[gameVersion]?.loaders ?? [];
    case ModLoader.Quilt:
      return (
        loaderVersions.quilt?.gameVersions[dummyReplaceString].loaders ?? []
      );
    default:
      return [];
  }
};
