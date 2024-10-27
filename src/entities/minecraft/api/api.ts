import { invoke } from '@tauri-apps/api/core';

import {
  VersionManifest,
  InstanceCreateDto,
  Instance,
  LoadingBar,
  MinecraftProcessMetadata,
} from '../model';

export const initializeState = () => invoke('initialize_state');

export const getMinecraftVersionManifest = () =>
  invoke<VersionManifest>('get_minecraft_version_manifest');

export const createMinecraftInstance = (instanceCreateDto: InstanceCreateDto) =>
  invoke<string>('create_minecraft_instance', { instanceCreateDto });

export const getMinecraftInstances = () =>
  invoke<Instance[]>('get_minecraft_instances');

export const launchMinecraftInstance = (nameId: string) =>
  invoke('launch_minecraft_instance', { nameId });

export const stopMinecraftInstance = (uuid: string) =>
  invoke('stop_minecraft_instance', { uuid });

export const removeMinecraftInstance = (nameId: string) =>
  invoke('remove_minecraft_instance', { nameId });

export const getLoadingBars = () =>
  invoke<Record<string, LoadingBar>>('get_progress_bars');

export const getRunningMinecraftInstances = () =>
  invoke<MinecraftProcessMetadata[]>('get_running_minecraft_instances');

export const getMinecraftInstanceProcess = (nameId: string) =>
  invoke<MinecraftProcessMetadata[]>('get_running_minecraft_instances', {
    nameId,
  });
