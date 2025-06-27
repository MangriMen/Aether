import type { ModLoader } from '../model';

export const QUERY_KEYS = {
  MINECRAFT: {
    SELF: ['minecraft'],
    MINECRAFT_VERSION_MANIFEST: () => [
      ...QUERY_KEYS.MINECRAFT.SELF,
      'minecraft_version_manifest',
    ],
    LOADER_VERSION_MANIFEST: (loader: ModLoader) => [
      ...QUERY_KEYS.MINECRAFT.SELF,
      'loader_version_manifest',
      loader,
    ],
  },
} as const;
