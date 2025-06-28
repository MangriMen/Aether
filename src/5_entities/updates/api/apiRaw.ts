import type { CheckOptions } from '@tauri-apps/plugin-updater';
import { check } from '@tauri-apps/plugin-updater';

export const checkUpdateRaw = (options?: CheckOptions) => check(options);
