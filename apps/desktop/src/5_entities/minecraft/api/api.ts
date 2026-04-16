import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { commands } from '@/shared/api/bindings/minecraft';

import type { ModLoader } from '../model';

import { QUERY_KEYS } from './queryKeys';

export const useMinecraftVersionManifest = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.MINECRAFT.MINECRAFT_VERSION_MANIFEST(),
    queryFn: commands.getMinecraftVersionManifest,
  }));

export const useLoaderVersionManifest = (loader: Accessor<ModLoader>) =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.MINECRAFT.LOADER_VERSION_MANIFEST(loader()),
    queryFn: () => commands.getLoaderVersionManifest(loader()),
  }));
