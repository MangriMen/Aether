import { invoke } from '@tauri-apps/api/core';

import type { AppSettings, UpdateAppSettings } from '../model';

export const getAppSettingsRaw = () => invoke<AppSettings>('get_app_settings');

export const updateAppSettingsRaw = (updateAppSettings: UpdateAppSettings) =>
  invoke('update_app_settings', { updateAppSettings });
