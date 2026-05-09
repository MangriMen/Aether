import { invoke } from '@tauri-apps/api/core';

export const revealInExplorerRaw = (path: string, exact = true) =>
  invoke('reveal_in_explorer', { path, exact });
