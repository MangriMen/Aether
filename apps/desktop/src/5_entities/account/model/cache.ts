import type { QueryClient } from '@tanstack/solid-query';

import { accountKeys } from './queryKeys';

export const accountInvalidation = {
  all: (queryClient: QueryClient) =>
    queryClient.invalidateQueries({ queryKey: accountKeys.all }),
  list: (queryClient: QueryClient) =>
    queryClient.invalidateQueries({ queryKey: accountKeys.list() }),
} as const;
