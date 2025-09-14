import { invoke } from '@tauri-apps/api/core';

export const callPlugin = (id: string, data: string) =>
  invoke('call_plugin', { data, id });
