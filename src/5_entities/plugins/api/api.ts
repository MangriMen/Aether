import { invoke } from '@tauri-apps/api/core';
import type { PluginInfo, PluginMetadata, PluginSettings } from '../model';

const PLUGIN_PREFIX = 'plugin:plugin|';

export const syncPlugins = async () => invoke(`${PLUGIN_PREFIX}sync_plugins`);

export const listPlugins = async () =>
  invoke<PluginMetadata[]>(`${PLUGIN_PREFIX}list_plugins`);

export const getPlugin = async (id: PluginInfo['id']) =>
  invoke<PluginMetadata>(`${PLUGIN_PREFIX}plugin_get`, { id });

export const enablePlugin = async (id: PluginInfo['id']) =>
  invoke(`${PLUGIN_PREFIX}enable_plugin`, { id });

export const disablePlugin = async (id: PluginInfo['id']) =>
  invoke(`${PLUGIN_PREFIX}disable_plugin`, { id });

export const getIsPluginEnabled = async (id: PluginInfo['id']) =>
  invoke<boolean>(`${PLUGIN_PREFIX}is_plugin_enabled`, { id });

export const getPluginSettings = async (id: PluginInfo['id']) =>
  invoke<PluginSettings | undefined>(`${PLUGIN_PREFIX}plugin_get_settings`, {
    id,
  });

export const editPluginSettings = async (
  id: PluginInfo['id'],
  settings: PluginSettings,
) => invoke(`${PLUGIN_PREFIX}plugin_edit_settings`, { id, settings });

export const openPluginsFolder = async () =>
  invoke(`${PLUGIN_PREFIX}open_plugins_folder`);
