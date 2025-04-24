import { invoke } from '@tauri-apps/api/core';

import type {
  ContentRequest,
  ImportHandler,
  Instance,
  NewInstance,
  InstanceFile,
  InstanceImportDto,
  MinecraftProcessMetadata,
  ContentResponse,
  InstallContentPayload,
  ContentType,
  EditInstance,
} from '../model';

const PLUGIN_INSTANCE_PREFIX = 'plugin:instance|';

const PLUGIN_PROCESS_PREFIX = 'plugin:process|';

export const createInstance = (newInstance: NewInstance) =>
  invoke<string>(`${PLUGIN_INSTANCE_PREFIX}instance_create`, {
    newInstance,
  });

export const installInstance = (id: Instance['id'], force: boolean = false) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_install`, { id, force });

export const updateInstance = (id: Instance['id']) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_update`, { id });

export const importInstance = (instanceImportDto: InstanceImportDto) =>
  invoke<string>(`${PLUGIN_INSTANCE_PREFIX}instance_import`, {
    instanceImportDto,
  });

export const getImportConfigs = () =>
  invoke<ImportHandler[]>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_import_configs`,
  );

export const listInstances = () =>
  invoke<Instance[]>(`${PLUGIN_INSTANCE_PREFIX}instance_list`);

export const getInstance = (id: Instance['id']) =>
  invoke<Instance>(`${PLUGIN_INSTANCE_PREFIX}instance_get`, { id });

export const launchInstance = (id: Instance['id']) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_launch`, { id });

export const stopInstance = (uuid: string) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_stop`, { uuid });

export const removeInstance = (id: Instance['id']) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_remove`, { id });

export const editInstance = async (
  id: Instance['id'],
  editInstance: EditInstance,
) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_edit`, {
    id,
    instanceEditDto: editInstance,
  });

// Utils
export const revealInExplorer = (path: string, exact = true) =>
  invoke('reveal_in_explorer', { path, exact });

// Process
export const listProcess = () =>
  invoke<MinecraftProcessMetadata[]>(`${PLUGIN_PROCESS_PREFIX}process_list`);

export const getInstanceProcess = (id: string) =>
  invoke<MinecraftProcessMetadata[]>(
    `${PLUGIN_PROCESS_PREFIX}process_get_by_instance_id`,
    {
      id,
    },
  );

// Content
export const getInstanceContents = (id: string) =>
  invoke<Record<string, InstanceFile>>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_contents`,
    { id },
  );

export const toggleDisableInstanceContent = (id: string, contentPath: string) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_toggle_disable_content`, {
    id,
    contentPath,
  });

export const disableInstanceContents = (id: string, contentPaths: string[]) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_disable_contents`, {
    id,
    contentPaths,
  });

export const enableInstanceContents = (id: string, contentPaths: string[]) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_enable_contents`, {
    id,
    contentPaths,
  });

export const removeInstanceContent = (id: string, contentPath: string) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_remove_content`, {
    id,
    contentPath,
  });

export const removeInstanceContents = (id: string, contentPaths: string[]) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_remove_contents`, {
    id,
    contentPaths,
  });

export const getContentProviders = () =>
  invoke<Record<string, string>>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_content_providers`,
  );

export const getContentByProvider = (payload: ContentRequest) =>
  invoke<ContentResponse>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_content_by_provider`,
    { payload },
  );

export const installContent = (id: string, payload: InstallContentPayload) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_install_content`, {
    id,
    payload,
  });

export const getMetadataFieldToCheckInstalled = (provider: string) =>
  invoke<string>(
    `${PLUGIN_INSTANCE_PREFIX}instance_get_metadata_field_to_check_installed`,
    { provider },
  );

export const importContents = (
  id: string,
  paths: string[],
  contentType: ContentType,
) =>
  invoke(`${PLUGIN_INSTANCE_PREFIX}instance_import_contents`, {
    id,
    paths,
    contentType,
  });
