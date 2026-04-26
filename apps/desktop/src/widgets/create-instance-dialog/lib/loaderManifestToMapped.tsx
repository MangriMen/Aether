import type {
  ModdedManifest,
  MappedLoaderManifest,
} from '@/entities/minecraft';

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
  };
};
