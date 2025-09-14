import { useQuery } from '@tanstack/solid-query';
import { UPDATE_QUERY_KEYS } from './queryKeys';
import { checkUpdateRaw } from './apiRaw';
import type { CheckOptions } from '@tauri-apps/plugin-updater';
import type { Accessor } from 'solid-js';

export const useCheckUpdate = (options?: Accessor<CheckOptions>) =>
  useQuery(() => ({
    queryKey: UPDATE_QUERY_KEYS.CHECK(),
    queryFn: () => checkUpdateRaw(options?.()),
  }));
