import type { Update } from '@tauri-apps/plugin-updater';

export const checkIsUpdateAvailable = (
  update: null | Update,
): update is Update => update !== null && update.version !== '';
