import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, EditAppSettings } from '../../model';

export const getAppSettingsRaw = () => invoke<AppSettings>('get_app_settings');

export const updateAppSettingsRaw = (updateAppSettings: EditAppSettings) =>
  invoke('update_app_settings', { updateAppSettings });
