import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import type { ModLoader } from '../model';

import { QUERY_KEYS } from './query_keys';
import {
  getLoaderVersionManifestRaw,
  getMinecraftVersionManifestRaw,
} from './rawApi';

export const useMinecraftVersionManifest = () =>
  useQuery(() => ({
    queryFn: getMinecraftVersionManifestRaw,
    queryKey: QUERY_KEYS.MINECRAFT.MINECRAFT_VERSION_MANIFEST(),
  }));

export const useLoaderVersionManifest = (loader: Accessor<ModLoader>) =>
  useQuery(() => ({
    queryFn: () => getLoaderVersionManifestRaw(loader()),
    queryKey: QUERY_KEYS.MINECRAFT.LOADER_VERSION_MANIFEST(loader()),
  }));
