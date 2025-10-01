import { invoke } from '@tauri-apps/api/core';

import type {
  ContentRequest,
  ImportConfig,
  Instance,
  NewInstance,
  InstanceFile,
  ImportInstance,
  MinecraftProcessMetadata,
  ContentResponse,
  InstallContentPayload,
  ContentType,
  EditInstance,
} from '../model';

const PLUGIN_INSTANCE_PREFIX = 'plugin:instance|';

const PLUGIN_PROCESS_PREFIX = 'plugin:process|';

export const createInstanceRaw = (newInstance: NewInstance) =>
  invoke<string>(`${PLUGIN_INSTANCE_PREFIX}instance_create`, {
    newInstance,
  });

export const installInstanceRaw = (
  id: Instance['id'],
  force: boolean = false,
) => invoke(`${PLUGIN_INSTANCE_PREFIX}instance_install`, { id, force });

export const updateInstanceRaw = (id: Instance['id']) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_update`, { id });

export const importInstanceRaw = (importInstance: ImportInstance) =>
  invoke<string>(`${PLUGIN_INSTANCE_PREFIX}instance_import`, {
    importInstance,
  });

export const getImportConfigsRaw = () =>
  invoke<ImportConfig[]>(
    `${PLUGIN_INSTANCE_PREFIX}instance_list_import_configs`,
  );

export const listInstancesRaw = () =>
  invoke<Instance[]>(`${PLUGIN_INSTANCE_PREFIX}instance_list`);

export const getInstanceRaw = (id: Instance['id']) =>
  invoke<Instance>(`${PLUGIN_INSTANCE_PREFIX}instance_get`, { id });

export const launchInstanceRaw = (id: Instance['id']) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_launch`, { id });

export const stopInstanceRaw = (uuid: string) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_stop`, { uuid });

export const removeInstanceRaw = (id: Instance['id']) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_remove`, { id });

export const editInstanceRaw = async (
  id: Instance['id'],
  editInstance: EditInstance,
) =>
  invoke<Instance>(`${PLUGIN_INSTANCE_PREFIX}instance_edit`, {
    id,
    editInstance,
  });

// Utils
export const revealInExplorerRaw = (path: string, exact = true) =>
  invoke('reveal_in_explorer', { path, exact });

// Process
export const listProcessRaw = () =>
  invoke<MinecraftProcessMetadata[]>(`${PLUGIN_PROCESS_PREFIX}process_list`);

export const getInstanceProcessRaw = (id: string) =>
  invoke<MinecraftProcessMetadata[]>(
    `${PLUGIN_PROCESS_PREFIX}process_get_by_instance_id`,
    {
      id,
    },
  );

// Content
export const getInstanceContentsRaw = (id: string) =>
  invoke<Record<string, InstanceFile>>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_contents`,
    { id },
  );

export const disableInstanceContentsRaw = (
  id: string,
  contentPaths: string[],
) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_disable_contents`, {
    id,
    contentPaths,
  });

export const enableInstanceContentsRaw = (id: string, contentPaths: string[]) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_enable_contents`, {
    id,
    contentPaths,
  });

export const removeInstanceContentRaw = (id: string, contentPath: string) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_remove_content`, {
    id,
    contentPath,
  });

export const removeInstanceContentsRaw = (id: string, contentPaths: string[]) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_remove_contents`, {
    id,
    contentPaths,
  });

export const getContentProvidersRaw = () =>
  invoke<Record<string, string>>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_content_providers`,
  );

export const getContentByProviderRaw = (payload: ContentRequest) =>
  invoke<ContentResponse>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_content_by_provider`,
    { payload },
  );

export const installContentRaw = (id: string, payload: InstallContentPayload) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_install_content`, {
    id,
    payload,
  });

export const getMetadataFieldToCheckInstalledRaw = (provider: string) =>
  invoke<string>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_metadata_field_to_check_installed`,
    { provider },
  );

export const importContentsRaw = (
  instanceId: string,
  sourcePaths: string[],
  contentType: ContentType,
) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_import_contents`, {
    instanceId,
    sourcePaths,
    contentType,
  });

export const getInstanceDirRaw = (id: Instance['id']) =>
  invoke<string>(`${PLUGIN_INSTANCE_PREFIX}instance_get_dir`, { id });

export const testInvoke = () => invoke('test');
