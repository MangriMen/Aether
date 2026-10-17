import type {
  MappedLoaderManifest,
  Version,
  ModLoader,
} from '@/features/minecraft';

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

const getVersionLoaders = (
  manifest: MappedLoaderManifest | undefined,
  gameVersion: string,
) => {
  const version = manifest?.gameVersions[gameVersion];

  if (!version) {
    return [];
  }

  return version.versionGroup
    ? (manifest?.versionGroups[version.versionGroup]?.loaders ?? [])
    : version.loaders;
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
      return getVersionLoaders(loaderVersions.forge, gameVersion);
    case 'fabric':
      return getVersionLoaders(loaderVersions.fabric, dummyReplaceString);
    case 'quilt':
      return getVersionLoaders(loaderVersions.quilt, gameVersion);
    case 'neoforge':
      return getVersionLoaders(loaderVersions.neoforge, gameVersion);
    default:
      return [];
  }
};
