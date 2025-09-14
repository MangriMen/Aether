import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '../model';

export const getSettingsRaw = () => invoke<Settings>('get_settings');

export const getMaxRamRaw = () => invoke<number>('get_max_ram');
