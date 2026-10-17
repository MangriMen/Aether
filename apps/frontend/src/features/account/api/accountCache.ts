import type { QueryClient } from '@tanstack/solid-query';

import { accountKeys } from './accountQueryKeys';

export const accountCache = {
  invalidate: {
    all: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({ queryKey: accountKeys.all }),
    list: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({ queryKey: accountKeys.list() }),
  },
} as const;
