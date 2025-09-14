import { useQuery } from '@tanstack/solid-query';
import {
  getLoaderVersionManifestRaw,
  getMinecraftVersionManifestRaw,
} from './rawApi';
import { QUERY_KEYS } from './query_keys';
import type { Accessor } from 'solid-js';
import type { ModLoader } from '../model';

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
