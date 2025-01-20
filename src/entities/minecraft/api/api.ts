import { invoke } from '@tauri-apps/api/core';

import {
  VersionManifest,
  InstanceCreateDto,
  Instance,
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

export const createMinecraftInstance = (instanceCreateDto: InstanceCreateDto) =>
  invoke<string>('create_minecraft_instance', { instanceCreateDto });

export const getMinecraftInstances = () =>
  invoke<[Instance[], string[]]>('get_minecraft_instances');

export const launchMinecraftInstance = (id: string) =>
  invoke('launch_minecraft_instance', { id });

export const stopMinecraftInstance = (uuid: string) =>
  invoke('stop_minecraft_instance', { uuid });

export const removeMinecraftInstance = (id: string) =>
  invoke('remove_minecraft_instance', { id });

export const getLoadingBars = () =>
  invoke<Record<string, LoadingBar>>('get_progress_bars');

export const getRunningMinecraftInstances = () =>
  invoke<MinecraftProcessMetadata[]>('get_running_minecraft_instances');

export const getMinecraftInstanceProcess = (id: string) =>
  invoke<MinecraftProcessMetadata[]>('get_minecraft_instance_process', {
    id,
  });
