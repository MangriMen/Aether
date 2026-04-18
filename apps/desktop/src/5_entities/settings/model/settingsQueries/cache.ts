import type { QueryClient, Updater } from '@tanstack/solid-query';

import { commands, type SettingsDto } from '../../api';
import { settingsKeys } from './queryKeys';

export const settingsQueries = {
  get: () => ({
    queryKey: settingsKeys.get(),
    queryFn: commands.get,
  }),
  maxRam: () => ({
    queryKey: settingsKeys.maxRam(),
    queryFn: commands.getMaxRam,
    staleTime: Infinity,
  }),
} as const;

export const settingsCache = {
  invalidate: {
    all: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: settingsKeys.all,
      }),
    get: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: settingsKeys.get(),
      }),
  },
  set: (
    queryClient: QueryClient,
    updater: Updater<SettingsDto | undefined, SettingsDto>,
  ) => queryClient.setQueryData(settingsKeys.get(), updater),

  prefetch: {
    maxRam: (queryClient: QueryClient) =>
      queryClient.prefetchQuery(settingsQueries.maxRam()),
  },
} as const;
