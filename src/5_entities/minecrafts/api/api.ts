import { invoke } from '@tauri-apps/api/core';

import type { VersionManifest, LoaderManifest, ModLoader } from '../model';

export const initializeState = () => invoke('initialize_state');

export const initializePlugins = () => invoke('initialize_plugins');

export const getMinecraftVersionManifest = () =>
  invoke<VersionManifest>('get_minecraft_version_manifest');

export const getLoaderVersionManifest = (loader: ModLoader) =>
  invoke<LoaderManifest>('get_loader_version_manifest', { loader });
