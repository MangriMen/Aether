import { createPluginInvoke } from '@/shared/lib';

import type {
  EditPluginSettings,
  Importer,
  Plugin,
  PluginMetadata,
  PluginSettings,
} from '../model';

const invokePlugin = createPluginInvoke('plugin');

export const importPluginsRaw = async (paths: string[]) =>
  invokePlugin('import', { paths });

export const syncPluginsRaw = async () => invokePlugin(`sync`);

export const listPluginsRaw = async () => invokePlugin<Plugin[]>(`list`);

export const getPluginRaw = async (id: PluginMetadata['id']) =>
  invokePlugin<Plugin>(`get`, { id });

export const removePluginRaw = async (id: PluginMetadata['id']) =>
  invokePlugin('remove', { id });

export const enablePluginRaw = async (id: PluginMetadata['id']) =>
  invokePlugin(`enable`, { id });

export const disablePluginRaw = async (id: PluginMetadata['id']) =>
  invokePlugin(`disable`, { id });

export const getPluginSettingsRaw = async (id: PluginMetadata['id']) =>
  invokePlugin<PluginSettings | undefined>(`get_settings`, {
    id,
  });

export const editPluginSettingsRaw = async ({
  id,
  editSettings,
}: {
  id: PluginMetadata['id'];
  editSettings: EditPluginSettings;
}) => invokePlugin(`edit_settings`, { id, editSettings });

export const openPluginsFolderRaw = async () =>
  invokePlugin(`open_plugins_folder`);

export const listImportersRaw = () =>
  invokePlugin<Importer[]>('list_importers');

export const getApiVersion = () => invokePlugin<string>('get_api_version');
