import type { ModLoader } from '.';

export const minecraftKeys = {
  minecraft: {
    all: ['minecraft'] as const,
    minecraftVersionManifest: () =>
      [...minecraftKeys.minecraft.all, 'minecraft_version_manifest'] as const,
    loaderVersionManifest: (loader: ModLoader) =>
      [
        ...minecraftKeys.minecraft.all,
        'loader_version_manifest',
        loader,
      ] as const,
  },
} as const;
