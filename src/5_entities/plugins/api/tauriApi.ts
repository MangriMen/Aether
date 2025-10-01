import { createPluginInvoke } from '@/shared/lib';

import type { Plugin, PluginMetadata, PluginSettings } from '../model';

const invokePlugin = createPluginInvoke('plugin');

export const syncPluginsRaw = async () => invokePlugin(`sync`);

export const listPluginsRaw = async () => invokePlugin<Plugin[]>(`list`);

export const getPluginRaw = async (id: PluginMetadata['id']) =>
  invokePlugin<Plugin>(`get`, { id });

export const enablePluginRaw = async (id: PluginMetadata['id']) =>
  invokePlugin(`plugin`, { id });

export const disablePluginRaw = async (id: PluginMetadata['id']) =>
  invokePlugin(`plugin`, { id });

export const getPluginSettingsRaw = async (id: PluginMetadata['id']) =>
  invokePlugin<PluginSettings | undefined>(`get_settings`, {
    id,
  });

export const editPluginSettingsRaw = async ({
  id,
  settings,
}: {
  id: PluginMetadata['id'];
  settings: PluginSettings;
}) => invokePlugin(`edit_settings`, { id, settings });

export const openPluginsFolderRaw = async () =>
  invokePlugin(`open_plugins_folder`);
