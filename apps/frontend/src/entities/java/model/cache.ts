import type { QueryClient } from '@tanstack/solid-query';

import { javaKeys } from './queryKeys';

export const javaCache = {
  invalidate: {
    all: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: javaKeys.all,
      }),
    list: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: javaKeys.list(),
      }),
    getActiveInstallations: (queryClient: QueryClient) =>
      queryClient.invalidateQueries({
        queryKey: javaKeys.getActiveInstallations(),
      }),
  },
};
