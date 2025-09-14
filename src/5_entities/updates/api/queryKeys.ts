import type { CheckOptions } from '@tauri-apps/plugin-updater';

export const UPDATE_QUERY_KEYS = {
  CHECK: (options?: CheckOptions) => [
    UPDATE_QUERY_KEYS.SELF,
    'check',
    `${JSON.stringify(options)}`,
  ],
  SELF: 'update',
};
