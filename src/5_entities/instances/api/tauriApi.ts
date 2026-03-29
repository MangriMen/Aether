import { invoke } from '@tauri-apps/api/core';

import type { CapabilityEntry } from '@/shared/model';

import { createPluginInvoke } from '@/shared/lib';

import type {
  ContentSearchParams,
  Instance,
  NewInstance,
  ContentFile,
  ImportInstance,
  MinecraftProcessMetadata,
  ContentSearchResponse,
  InstallContentParams,
  ContentType,
  EditInstance,
  ContentItem,
  ContentVersion,
  ContentGetParams,
  ContentListVersionParams,
} from '../model';
import type { ContentProviderCapabilityMetadata } from '../model/capabilities';
import type {
  ContentCompatibilityCheckParams,
  ContentCompatibilityResult,
} from '../model/compatibility';

export const invokeInstance = createPluginInvoke('instance');
export const invokeProcess = createPluginInvoke('process');

export const createInstanceRaw = (newInstance: NewInstance) =>
  invokeInstance<string>(`create`, { newInstance });

export const installInstanceRaw = (
  id: Instance['id'],
  force: boolean = false,
) => invokeInstance(`install`, { id, force });

export const updateInstanceRaw = (id: Instance['id']) =>
  invokeInstance(`update`, { id });

export const importInstanceRaw = (importInstance: ImportInstance) =>
  invokeInstance<string>(`import`, {
    importInstance,
  });

export const listInstancesRaw = () => invokeInstance<Instance[]>(`list`);

export const getInstanceRaw = (id: Instance['id']) =>
  invokeInstance<Instance>(`get`, { id });

export const launchInstanceRaw = (id: Instance['id']) =>
  invokeInstance(`launch`, { id });

export const stopInstanceRaw = (uuid: string) =>
  invokeInstance(`stop`, { uuid });

export const removeInstanceRaw = (id: Instance['id']) =>
  invokeInstance(`remove`, { id });

export const editInstanceRaw = async (
  id: Instance['id'],
  editInstance: EditInstance,
) =>
  invokeInstance<Instance>(`edit`, {
    id,
    editInstance,
  });

// Utils
export const revealInExplorerRaw = (path: string, exact = true) =>
  invoke('reveal_in_explorer', { path, exact });

// Process
export const listProcessRaw = () =>
  invokeProcess<MinecraftProcessMetadata[]>(`list`);

export const getInstanceProcessRaw = (id: string) =>
  invokeProcess<MinecraftProcessMetadata[]>(`get_by_instance_id`, {
    id,
  });

// Content
export const listContentRaw = (id: string) =>
  invokeInstance<Record<string, ContentFile>>(`list_content`, { id });

export const disableContentRaw = (id: string, contentPaths: string[]) =>
  invokeInstance(`disable_contents`, {
    id,
    contentPaths,
  });

export const enableContentRaw = (id: string, contentPaths: string[]) =>
  invokeInstance(`enable_contents`, {
    id,
    contentPaths,
  });

export const removeContentsRaw = (id: string, contentPaths: string[]) =>
  invokeInstance(`remove_contents`, {
    id,
    contentPaths,
  });

export const listContentProvidersRaw = () =>
  invokeInstance<Array<CapabilityEntry<ContentProviderCapabilityMetadata>>>(
    `list_content_providers`,
  );

export const searchContentRaw = (payload: ContentSearchParams) =>
  invokeInstance<ContentSearchResponse>(`search_content`, {
    payload,
  });

export const installContentRaw = (payload: InstallContentParams) =>
  invokeInstance(`install_content`, {
    payload,
  });

export const importContentsRaw = (
  instanceId: string,
  sourcePaths: string[],
  contentType: ContentType,
) =>
  invokeInstance(`import_contents`, {
    instanceId,
    sourcePaths,
    contentType,
  });

export const getInstanceDirRaw = (id: Instance['id']) =>
  invokeInstance<string>(`get_dir`, { id });

export const checkCompatibility = (
  instanceIds: Array<Instance['id']>,
  checkParams: ContentCompatibilityCheckParams,
) =>
  invokeInstance<Record<Instance['id'], ContentCompatibilityResult>>(
    'check_compatibility',
    {
      instanceIds,
      checkParams,
    },
  );
export const getContentRaw = (params: ContentGetParams) =>
  invokeInstance<ContentItem>('get_content', { params });

export const listContentVersionRaw = (params: ContentListVersionParams) =>
  invokeInstance<ContentVersion[]>(`list_content_version`, { params });
