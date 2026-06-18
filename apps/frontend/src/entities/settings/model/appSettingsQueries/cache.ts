import type { QueryClient } from '@tanstack/solid-query';

import { settingsCommands } from '../../api';
import { appSettingsKeys } from './queryKeys';

export const appSettingsQueries = {
  get: () => ({
    queryKey: appSettingsKeys.get(),
    queryFn: settingsCommands.getAppSettings,
  }),
} as const;

export const appSettingsCache = {
  invalidate: {
    all: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
    get: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({ queryKey: appSettingsKeys.get() }),
  },
} as const;
