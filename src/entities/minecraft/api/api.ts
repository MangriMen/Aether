import { invoke } from '@tauri-apps/api/core';

import { VersionManifest, InstanceCreateDto, Instance } from '../model';

export const initializeState = () => invoke('initialize_state');

export const getMinecraftVersionManifest = () =>
  invoke<VersionManifest>('get_minecraft_version_manifest');

export const createMinecraftInstance = (instanceCreateDto: InstanceCreateDto) =>
  invoke<string>('create_minecraft_instance', { instanceCreateDto });

export const getMinecraftInstances = () =>
  invoke<Instance[]>('get_minecraft_instances');

export const launchMinecraftInstance = (name: string) =>
  invoke('launch_minecraft_instance', { name });
