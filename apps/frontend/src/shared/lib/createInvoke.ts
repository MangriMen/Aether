import type { InvokeArgs, InvokeOptions } from '@tauri-apps/api/core';

import { invoke } from '@tauri-apps/api/core';

export const createPluginInvoke = (plugin: string): typeof invoke =>
  createPrefixedInvoke(`plugin:${plugin}|`);

export const createPrefixedInvoke = (prefix: string): typeof invoke => {
  return <T>(
    cmd: string,
    args?: InvokeArgs,
    options?: InvokeOptions,
  ): Promise<T> => invoke<T>(`${prefix}${cmd}`, args, options);
};
