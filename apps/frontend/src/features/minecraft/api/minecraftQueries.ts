import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { minecraftCommands } from '@/platform/ipc';

import type { ModLoader } from '../model/modLoader';

import { minecraftKeys } from './minecraftQueryKeys';

export const useMinecraftVersionManifest = () =>
  useQuery(() => ({
    queryKey: minecraftKeys.minecraft.minecraftVersionManifest(),
    queryFn: minecraftCommands.getMinecraftVersionManifest,
  }));

export const useLoaderVersionManifest = (loader: Accessor<ModLoader>) =>
  useQuery(() => ({
    queryKey: minecraftKeys.minecraft.loaderVersionManifest(loader()),
    queryFn: () => minecraftCommands.getLoaderVersionManifest(loader()),
  }));
