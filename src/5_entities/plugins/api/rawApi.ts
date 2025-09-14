import { invoke } from '@tauri-apps/api/core';
import type { Plugin, PluginMetadata, PluginSettings } from '../model';

const PLUGIN_PREFIX = 'plugin:plugin|';

export const syncPluginsRaw = async () =>
  invoke(`${PLUGIN_PREFIX}sync_plugins`);

export const listPluginsRaw = async () =>
  invoke<Plugin[]>(`${PLUGIN_PREFIX}list_plugins`);

export const getPluginRaw = async (id: PluginMetadata['id']) =>
  invoke<Plugin>(`${PLUGIN_PREFIX}plugin_get`, { id });

export const enablePluginRaw = async (id: PluginMetadata['id']) =>
  invoke(`${PLUGIN_PREFIX}enable_plugin`, { id });

export const disablePluginRaw = async (id: PluginMetadata['id']) =>
  invoke(`${PLUGIN_PREFIX}disable_plugin`, { id });

export const getIsPluginEnabledRaw = async (id: PluginMetadata['id']) =>
  invoke<boolean>(`${PLUGIN_PREFIX}is_plugin_enabled`, { id });

export const getPluginSettingsRaw = async (id: PluginMetadata['id']) =>
  invoke<PluginSettings | undefined>(`${PLUGIN_PREFIX}plugin_get_settings`, {
    id,
  });

export const editPluginSettingsRaw = async ({
  id,
  settings,
}: {
  id: PluginMetadata['id'];
  settings: PluginSettings;
}) => invoke(`${PLUGIN_PREFIX}plugin_edit_settings`, { id, settings });

export const openPluginsFolderRaw = async () =>
  invoke(`${PLUGIN_PREFIX}open_plugins_folder`);
