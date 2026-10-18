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
  // Whether clearing is allowed changes by itself, as downloads start and end. That is told by
  // the cache event; it is asked again only on opening, in case an event was missed.
  cacheBusy: () => ({
    queryKey: settingsKeys.cacheBusy(),
    queryFn: settingsCommands.isCacheBusy,
    staleTime: Infinity,
    refetchOnMount: 'always' as const,
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
      queryClient.invalidateQueries({
        queryKey: settingsKeys.cacheUsage(),
      }),
  },
  set: (
    queryClient: QueryClient,
    updater: Updater<SettingsDto | undefined, SettingsDto>,
  ) => queryClient.setQueryData(settingsKeys.get(), updater),
  // An answer still on its way was asked before the event, so it must not overwrite it.
  setCacheBusy: async (queryClient: QueryClient, busy: boolean) => {
    await queryClient.cancelQueries({ queryKey: settingsKeys.cacheBusy() });
    queryClient.setQueryData(settingsKeys.cacheBusy(), busy);
  },

  prefetch: {
    maxRam: (queryClient: QueryClient) =>
      queryClient.prefetchQuery(settingsQueries.maxRam()),
  },
} as const;
