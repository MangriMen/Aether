import { invoke } from '@tauri-apps/api/core';

import type { VersionManifest, LoaderManifest, ModLoader } from '../model';

export const initializeState = () => invoke('initialize_state');

export const getMinecraftVersionManifest = () =>
  invoke<VersionManifest>('get_minecraft_version_manifest');

export const getLoaderVersionsManifest = (loader: ModLoader) =>
  invoke<LoaderManifest>('get_loader_versions_manifest', { loader });
