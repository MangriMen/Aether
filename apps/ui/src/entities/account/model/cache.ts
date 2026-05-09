import type { QueryClient } from '@tanstack/solid-query';

import { commands } from '../api';
import { accountKeys } from './queryKeys';

export const accountQueries = {
  list: () => ({
    queryKey: accountKeys.list(),
    queryFn: commands.listAccounts,
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
