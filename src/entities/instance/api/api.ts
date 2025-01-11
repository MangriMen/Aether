import { invoke } from '@tauri-apps/api/core';

// eslint-disable-next-line boundaries/element-types
import { Instance } from '@/entities/minecraft';

export const openInstanceFolder = (instance: Instance, exact = true) =>
  invoke('reveal_in_explorer', { path: instance.path, exact });
