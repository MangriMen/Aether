import { invoke } from '@tauri-apps/api/core';

import type { ProgressBarDto } from '@/shared/api';

export const listProgressBarsRaw = () =>
  invoke<Array<ProgressBarDto>>('list_progress_bars');
