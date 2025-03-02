import { invoke } from '@tauri-apps/api/core';

import type {
  ImportHandler,
  Instance,
  InstanceCreateDto,
  InstanceEditDto,
  InstanceFile,
  InstanceImportDto,
  MinecraftProcessMetadata,
} from '../model';

export const createInstance = (instanceCreateDto: InstanceCreateDto) =>
  invoke<string>('plugin:instance|instance_create', { instanceCreateDto });

export const importInstance = (instanceImportDto: InstanceImportDto) =>
  invoke<string>('plugin:instance|instance_import', { instanceImportDto });

export const getImportConfigs = () =>
  invoke<ImportHandler[]>('plugin:instance|instance_get_import_configs');

export const listInstances = () =>
  invoke<[Instance[], string[]]>('plugin:instance|instance_list');

export const getInstance = (id: Instance['id']) =>
  invoke<Instance>('plugin:instance|instance_get', { id });

export const launchInstance = (id: Instance['id']) =>
  invoke('plugin:instance|instance_launch', { id });

export const stopInstance = (uuid: string) =>
  invoke('plugin:instance|instance_stop', { uuid });

export const removeInstance = (id: Instance['id']) =>
  invoke('plugin:instance|instance_remove', { id });

export const editInstance = async (
  id: Instance['id'],
  instanceEditDto: InstanceEditDto,
) => invoke('plugin:instance|instance_edit', { id, instanceEditDto });

export const revealInExplorer = (path: string, exact = true) =>
  invoke('reveal_in_explorer', { path, exact });

export const listProcess = () =>
  invoke<MinecraftProcessMetadata[]>('plugin:process|process_list');

export const getInstanceProcess = (id: string) =>
  invoke<MinecraftProcessMetadata[]>(
    'plugin:process|process_get_by_instance_id',
    {
      id,
    },
  );

export const getInstanceContents = (id: string) =>
  invoke<Record<string, InstanceFile>>(
    'plugin:instance|instance_get_contents',
    {
      id,
    },
  );

export const toggleDisableInstanceContent = (id: string, contentPath: string) =>
  invoke('plugin:instance|instance_toggle_disable_content', {
    id,
    contentPath,
  });

export const removeInstanceContent = (id: string, contentPath: string) =>
  invoke('plugin:instance|instance_remove_content', { id, contentPath });
