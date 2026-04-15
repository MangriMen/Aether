import { invoke } from '@tauri-apps/api/core';

import type { ModdedManifestDto, VersionManifestDto } from '@/shared/api';

import { createPluginInvoke } from '@/shared/lib';

import type { ModLoader } from '../model';

export const initializeState = () => invoke('initialize_state');

export const initializePlugins = () => invoke('initialize_plugins');

const invokeMinecraft = createPluginInvoke('minecraft');

export const getMinecraftVersionManifestRaw = () =>
  invokeMinecraft<VersionManifestDto>('get_minecraft_version_manifest');

export const getLoaderVersionManifestRaw = (loader: ModLoader) =>
  invokeMinecraft<ModdedManifestDto>('get_loader_version_manifest', { loader });
