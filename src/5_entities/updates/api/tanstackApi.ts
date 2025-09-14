import type { CheckOptions } from '@tauri-apps/plugin-updater';
import type { Accessor } from 'solid-js';

import { useQuery } from '@tanstack/solid-query';

import { checkUpdateRaw } from './apiRaw';
import { UPDATE_QUERY_KEYS } from './queryKeys';

export const useCheckUpdate = (options?: Accessor<CheckOptions>) =>
  useQuery(() => ({
    queryFn: () => checkUpdateRaw(options?.()),
    queryKey: UPDATE_QUERY_KEYS.CHECK(),
  }));
