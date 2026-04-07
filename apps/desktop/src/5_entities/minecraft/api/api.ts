import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import type { ModLoader } from '../model';

import { QUERY_KEYS } from './queryKeys';
import {
  getLoaderVersionManifestRaw,
  getMinecraftVersionManifestRaw,
} from './rawApi';

export const useMinecraftVersionManifest = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.MINECRAFT.MINECRAFT_VERSION_MANIFEST(),
    queryFn: getMinecraftVersionManifestRaw,
  }));

export const useLoaderVersionManifest = (loader: Accessor<ModLoader>) =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.MINECRAFT.LOADER_VERSION_MANIFEST(loader()),
    queryFn: () => getLoaderVersionManifestRaw(loader()),
  }));
