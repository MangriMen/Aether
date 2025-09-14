import { invoke } from '@tauri-apps/api/core';

import type { LoaderManifest, ModLoader, VersionManifest } from '../model';

export const initializeState = () => invoke('initialize_state');

export const initializePlugins = () => invoke('initialize_plugins');

export const getMinecraftVersionManifestRaw = () =>
  invoke<VersionManifest>('get_minecraft_version_manifest');

export const getLoaderVersionManifestRaw = (loader: ModLoader) =>
  invoke<LoaderManifest>('get_loader_version_manifest', { loader });
