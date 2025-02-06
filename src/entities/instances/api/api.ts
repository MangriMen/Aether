import { invoke } from '@tauri-apps/api/core';

import type {
  Instance,
  InstanceCreateDto,
  InstanceEditDto,
  MinecraftProcessMetadata,
} from '../model';

export const createMinecraftInstance = (instanceCreateDto: InstanceCreateDto) =>
  invoke<string>('create_minecraft_instance', { instanceCreateDto });

export const getMinecraftInstances = () =>
  invoke<[Instance[], string[]]>('get_minecraft_instances');

export const getMinecraftInstance = (id: Instance['id']) =>
  invoke<Instance>('get_minecraft_instance', { id });

export const launchMinecraftInstance = (id: Instance['id']) =>
  invoke('launch_minecraft_instance', { id });

export const stopMinecraftInstance = (uuid: string) =>
  invoke('stop_minecraft_instance', { uuid });

export const removeMinecraftInstance = (id: Instance['id']) =>
  invoke('remove_minecraft_instance', { id });

export const editMinecraftInstance = async (
  id: Instance['id'],
  instanceEditDto: InstanceEditDto,
) => invoke('edit_minecraft_instance', { id, instanceEditDto });

export const openInstanceFolder = (instance: Instance, exact = true) =>
  invoke('reveal_in_explorer', { path: instance.path, exact });

export const getRunningMinecraftInstances = () =>
  invoke<MinecraftProcessMetadata[]>('get_running_minecraft_instances');

export const getMinecraftInstanceProcess = (id: string) =>
  invoke<MinecraftProcessMetadata[]>('get_minecraft_instance_process', {
    id,
  });
