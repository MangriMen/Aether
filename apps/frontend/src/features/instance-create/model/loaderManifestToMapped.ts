import type {
  ModdedManifest,
  MappedLoaderManifest,
} from '@/features/minecraft';

export const loaderManifestToMapped = (
  manifest: ModdedManifest,
): MappedLoaderManifest => {
  return {
    gameVersions: manifest.gameVersions.reduce<
      MappedLoaderManifest['gameVersions']
    >((acc, version) => {
      acc[version.id] = version;
      return acc;
    }, {}),
    versionGroups: Object.fromEntries(
      manifest.versionGroups.map((group) => [group.id, group]),
    ),
  };
};
