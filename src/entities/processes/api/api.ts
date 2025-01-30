import { invoke } from '@tauri-apps/api/core';
import type { MinecraftProcessMetadata } from '../model';

export const getRunningMinecraftInstances = () =>
  invoke<MinecraftProcessMetadata[]>('get_running_minecraft_instances');

export const getMinecraftInstanceProcess = (id: string) =>
  invoke<MinecraftProcessMetadata[]>('get_minecraft_instance_process', {
    id,
  });
