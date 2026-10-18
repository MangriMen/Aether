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
    // Measuring walks the caches. What changes them is told by the cache event.
    staleTime: 30_000,
  }),
  // Whether clearing is allowed changes by itself, as downloads start and end. It is cheap,
  // so it is asked again whenever the pane is opened or the window gets focus.
  cacheBusy: () => ({
    queryKey: settingsKeys.cacheBusy(),
    queryFn: settingsCommands.isCacheBusy,
    staleTime: 0,
    refetchOnMount: 'always' as const,
    refetchOnWindowFocus: true,
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
    cacheUsage: (queryClient: QueryClient) =>
      Promise.all([
        queryClient.invalidateQueries({
          queryKey: settingsKeys.cacheUsage(),
        }),
        queryClient.invalidateQueries({
          queryKey: settingsKeys.cacheBusy(),
        }),
      ]),
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
