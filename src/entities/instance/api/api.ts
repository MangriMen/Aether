import { invoke } from '@tauri-apps/api/core';

import {
  Instance,
  InstanceCreateDto,
  InstanceEditDto,
  refetchInstances,
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
) => {
  await invoke('edit_minecraft_instance', { id, instanceEditDto });
  // await new Promise((r) => setTimeout(r, 10));
  refetchInstances();
};

export const openInstanceFolder = (instance: Instance, exact = true) =>
  invoke('reveal_in_explorer', { path: instance.path, exact });
