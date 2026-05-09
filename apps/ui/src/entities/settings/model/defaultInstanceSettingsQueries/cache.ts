import type { QueryClient, Updater } from '@tanstack/solid-query';

import { commands, type DefaultInstanceSettingsDto } from '../../api';
import { defaultInstanceSettingsKeys } from './queryKeys';

export const defaultInstanceSettingsQueries = {
  get: () => ({
    queryKey: defaultInstanceSettingsKeys.get(),
    queryFn: commands.getDefaultInstanceSettings,
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
