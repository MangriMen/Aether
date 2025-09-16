import type { InvokeArgs, InvokeOptions } from '@tauri-apps/api/core';

import { invoke } from '@tauri-apps/api/core';

const PLUGIN_SETTINGS_PREFIX = 'plugin:settings|';

export const invokeSettings: typeof invoke = <T>(
  cmd: string,
  args?: InvokeArgs,
  options?: InvokeOptions,
): Promise<T> => invoke<T>(`${PLUGIN_SETTINGS_PREFIX}${cmd}`, args, options);
