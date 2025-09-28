import type { AppSettings, EditAppSettings } from '../../model/appSettings';

import { invokeSettings } from '../tauriApi';

export const getAppSettingsRaw = () =>
  invokeSettings<AppSettings>('get_app_settings');

export const editAppSettingsRaw = (editAppSettings: EditAppSettings) =>
  invokeSettings('edit_app_settings', { editAppSettings });
