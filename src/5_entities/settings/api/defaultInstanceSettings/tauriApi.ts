import type {
  EditDefaultInstanceSettings,
  DefaultInstanceSettings,
} from '../../model/defaultInstanceSettings';

import { invokeSettings } from '../tauriApi';

export const getDefaultInstanceSettingsRaw = () =>
  invokeSettings<DefaultInstanceSettings>(`get_default_instance_settings`);

export const editDefaultInstanceSettingsRaw = (
  editSettings: EditDefaultInstanceSettings,
) =>
  invokeSettings<DefaultInstanceSettings>(`edit_default_instance_settings`, {
    editSettings,
  });
