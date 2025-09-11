import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '../model';
import type {
  EditDefaultInstanceSettings,
  DefaultInstanceSettings,
} from '../model/defaultInstanceSettings';

const PLUGIN_SETTINGS_PREFIX = 'plugin:settings|';

export const getSettingsRaw = () =>
  invoke<Settings>(`${PLUGIN_SETTINGS_PREFIX}get`);

export const editSettingsRaw = (editSettings: Settings) =>
  invoke<Settings>(`${PLUGIN_SETTINGS_PREFIX}edit`, { editSettings });

export const getMaxRamRaw = () =>
  invoke<number>(`${PLUGIN_SETTINGS_PREFIX}get_max_ram`);

export const getDefaultInstanceSettingsRaw = () =>
  invoke<DefaultInstanceSettings>(
    `${PLUGIN_SETTINGS_PREFIX}get_default_instance_settings`,
  );

export const editDefaultInstanceSettingsRaw = (
  editSettings: EditDefaultInstanceSettings,
) =>
  invoke<DefaultInstanceSettings>(
    `${PLUGIN_SETTINGS_PREFIX}edit_default_instance_settings`,
    {
      editSettings,
    },
  );
