import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '../model';

export const getMaxRam = () => invoke<number>('get_max_ram');

export const getSettings = () => invoke<Settings>('get_settings');
