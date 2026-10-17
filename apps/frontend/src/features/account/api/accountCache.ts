import type { QueryClient } from '@tanstack/solid-query';

import { authCommands } from '@/platform/ipc';

import { accountKeys } from './accountQueryKeys';

export const accountQueries = {
  list: () => ({
    queryKey: accountKeys.list(),
    queryFn: authCommands.listAccounts,
    reconcile: 'id',
    staleTime: Infinity,
  }),
} as const;

export const accountCache = {
  invalidate: {
    all: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({ queryKey: accountKeys.all }),
    list: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({ queryKey: accountKeys.list() }),
  },
} as const;
