import type { Update } from '@tauri-apps/plugin-updater';

export const checkIsUpdateAvailable = (
  update: Update | null,
): update is Update => update !== null && update.version !== '';
