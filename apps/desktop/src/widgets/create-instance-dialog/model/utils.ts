import type { MappedLoaderManifest, Version } from '@/entities/minecraft';
import type { ModLoader } from '@/entities/minecraft';

export const filterGameVersions = (
  versions: Version[],
  shouldIncludeSnapshots?: boolean,
) =>
  versions.filter(
    (version) => version.type === 'release' || shouldIncludeSnapshots,
  );

export const filterGameVersionsForLoader = (
  loader: ModLoader | undefined,
  versions: Version[],
  loaderVersions: {
    fabric?: MappedLoaderManifest;
    forge?: MappedLoaderManifest;
    quilt?: MappedLoaderManifest;
    neoforge?: MappedLoaderManifest;
  },
) => {
  if (!loader || loader === 'vanilla') {
    return versions;
  }

  return versions.filter((version) => {
    switch (loader) {
      case 'fabric':
        return !!loaderVersions.fabric?.gameVersions[version.id];
      case 'forge':
        return !!loaderVersions.forge?.gameVersions[version.id];
      case 'quilt':
        return !!loaderVersions.quilt?.gameVersions[version.id];
      case 'neoforge':
        return !!loaderVersions.neoforge?.gameVersions[version.id];
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
    neoforge?: MappedLoaderManifest;
  },
) => {
  if (!loader || !gameVersion) {
    return [];
  }

  const dummyReplaceString = '${modrinth.gameVersion}';
  switch (loader) {
    case 'forge':
      return loaderVersions.forge?.gameVersions[gameVersion]?.loaders ?? [];
    case 'fabric':
      return (
        loaderVersions.fabric?.gameVersions[dummyReplaceString]?.loaders ?? []
      );
    case 'quilt':
      return (
        loaderVersions.quilt?.gameVersions[dummyReplaceString]?.loaders ?? []
      );
    case 'neoforge':
      return loaderVersions.neoforge?.gameVersions[gameVersion]?.loaders ?? [];
    default:
      return [];
  }
};
