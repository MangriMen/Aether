import { check } from '@tauri-apps/plugin-updater';

export const getUpdate = () => check().catch(() => undefined);
