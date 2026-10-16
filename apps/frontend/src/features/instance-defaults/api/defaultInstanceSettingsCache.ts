import type { QueryClient, Updater } from '@tanstack/solid-query';

import { type DefaultInstanceSettingsDto } from '@/platform/bindings/settings';
import { settingsCommands } from '@/platform/ipc';

import { defaultInstanceSettingsKeys } from './defaultInstanceSettingsKeys';

export const defaultInstanceSettingsQueries = {
  get: () => ({
    queryKey: defaultInstanceSettingsKeys.get(),
    queryFn: settingsCommands.getDefaultInstanceSettings,
  }),
} as const;

export const defaultInstanceSettingsCache = {
  invalidate: {
    all: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: defaultInstanceSettingsKeys.all,
      }),
    get: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: defaultInstanceSettingsKeys.get(),
      }),
  },
  set: (
    queryClient: QueryClient,
    updater: Updater<
      DefaultInstanceSettingsDto | undefined,
      DefaultInstanceSettingsDto
    >,
  ) => queryClient.setQueryData(defaultInstanceSettingsKeys.get(), updater),
} as const;
