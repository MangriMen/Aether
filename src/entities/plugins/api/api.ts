import { invoke } from '@tauri-apps/api/core';
import type { PluginInfo, PluginMetadata, PluginSettings } from '../model';

export const scanPlugins = async () => invoke('scan_plugins');

export const listPlugins = async () => invoke<PluginMetadata[]>('list_plugins');

export const enablePlugin = async (id: PluginInfo['id']) =>
  invoke('enable_plugin', { id });

export const disablePlugin = async (id: PluginInfo['id']) =>
  invoke('disable_plugin', { id });

export const getIsPluginEnabled = async (id: PluginInfo['id']) =>
  invoke<boolean>('is_plugin_enabled', { id });

export const getPluginSettings = async (id: PluginInfo['id']) =>
  invoke<PluginSettings>('plugin_get_settings', { id });

export const editPluginSettings = async (
  id: PluginInfo['id'],
  settings: PluginSettings,
) => invoke('plugin_edit_settings', { id, settings });
