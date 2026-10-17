import type { QueryClient, Updater } from '@tanstack/solid-query';

import { type SettingsDto } from '@/platform/bindings/settings';
import { settingsCommands } from '@/platform/ipc';

import { settingsKeys } from './settingsQueryKeys';

export const settingsQueries = {
  get: () => ({
    queryKey: settingsKeys.get(),
    queryFn: settingsCommands.get,
  }),
  // Measuring walks the folders, so it is asked on opening, not on a timer.
  storageUsage: () => ({
    queryKey: settingsKeys.storageUsage(),
    queryFn: settingsCommands.getStorageUsage,
    staleTime: 30_000,
  }),
  cacheUsage: () => ({
    queryKey: settingsKeys.cacheUsage(),
    queryFn: settingsCommands.getCacheUsage,
    // Whether clearing is allowed changes by itself, as downloads start and end. Sizes are
    // measured on each poll, so it is slow; to be replaced by launcher events or the file watcher.
    refetchInterval: 10_000,
  }),
  maxRam: () => ({
    queryKey: settingsKeys.maxRam(),
    queryFn: settingsCommands.getMaxRam,
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
