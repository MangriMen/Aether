import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '../model';
import type {
  EditGlobalInstanceSettings,
  GlobalInstanceSettings,
} from '../model/globalInstanceSettings';

const PLUGIN_SETTINGS_PREFIX = 'plugin:settings|';

export const getSettingsRaw = () =>
  invoke<Settings>(`${PLUGIN_SETTINGS_PREFIX}get`);

export const editSettingsRaw = (editSettings: Settings) =>
  invoke<Settings>(`${PLUGIN_SETTINGS_PREFIX}edit`, { editSettings });

export const getMaxRamRaw = () =>
  invoke<number>(`${PLUGIN_SETTINGS_PREFIX}get_max_ram`);

export const getGlobalInstanceSettingsRaw = () =>
  invoke<GlobalInstanceSettings>(
    `${PLUGIN_SETTINGS_PREFIX}get_global_instance_settings`,
  );

export const editGlobalInstanceSettingsRaw = (
  editSettings: EditGlobalInstanceSettings,
) =>
  invoke<GlobalInstanceSettings>(
    `${PLUGIN_SETTINGS_PREFIX}edit_global_instance_settings`,
    {
      editSettings,
    },
  );
