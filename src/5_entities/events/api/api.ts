import { invoke } from '@tauri-apps/api/core';
import type { LoadingBar } from '../model';

export const getLoadingBars = () =>
  invoke<Record<string, LoadingBar>>('get_progress_bars');
