import type { EditSettings, Settings } from '../../model';

import { invokeSettings } from '../tauriApiRaw';

export const getSettingsRaw = () => invokeSettings<Settings>(`get`);

export const editSettingsRaw = (editSettings: EditSettings) =>
  invokeSettings<Settings>(`edit`, { editSettings });

export const getMaxRamRaw = () => invokeSettings<number>(`get_max_ram`);
