import type { ModLoader } from '../model';

export const QUERY_KEYS = {
  MINECRAFT: {
    LOADER_VERSION_MANIFEST: (loader: ModLoader) => [
      ...QUERY_KEYS.MINECRAFT.SELF,
      'loader_version_manifest',
      loader,
    ],
    MINECRAFT_VERSION_MANIFEST: () => [
      ...QUERY_KEYS.MINECRAFT.SELF,
      'minecraft_version_manifest',
    ],
    SELF: ['minecraft'],
  },
} as const;
