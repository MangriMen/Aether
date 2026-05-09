import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import type { ModLoader } from '.';

import { commands } from '../api';
import { minecraftKeys } from './queryKeys';

export const useMinecraftVersionManifest = () =>
  useQuery(() => ({
    queryKey: minecraftKeys.minecraft.minecraftVersionManifest(),
    queryFn: commands.getMinecraftVersionManifest,
  }));

export const useLoaderVersionManifest = (loader: Accessor<ModLoader>) =>
  useQuery(() => ({
    queryKey: minecraftKeys.minecraft.loaderVersionManifest(loader()),
    queryFn: () => commands.getLoaderVersionManifest(loader()),
  }));
