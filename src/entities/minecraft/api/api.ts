import { invoke } from '@tauri-apps/api/core';

import {
  VersionManifest,
  LoadingBar,
  MinecraftProcessMetadata,
  LoaderManifest,
  ModLoader,
} from '../model';

export const initializeState = () => invoke('initialize_state');

export const getMinecraftVersionManifest = () =>
  invoke<VersionManifest>('get_minecraft_version_manifest');

export const getLoaderVersionsManifest = (loader: ModLoader) =>
  invoke<LoaderManifest>('get_loader_versions_manifest', { loader });

export const getLoadingBars = () =>
  invoke<Record<string, LoadingBar>>('get_progress_bars');

export const getRunningMinecraftInstances = () =>
  invoke<MinecraftProcessMetadata[]>('get_running_minecraft_instances');

export const getMinecraftInstanceProcess = (id: string) =>
  invoke<MinecraftProcessMetadata[]>('get_minecraft_instance_process', {
    id,
  });
