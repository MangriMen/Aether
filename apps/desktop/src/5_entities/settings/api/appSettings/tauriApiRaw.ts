import type { AppSettingsDto, EditAppSettingsDto } from '@/shared/api';

import { invokeSettings } from '../tauriApi';

export const getAppSettingsRaw = () =>
  invokeSettings<AppSettingsDto>('get_app_settings');

export const editAppSettingsRaw = (editAppSettings: EditAppSettingsDto) =>
  invokeSettings('edit_app_settings', { editAppSettings });
