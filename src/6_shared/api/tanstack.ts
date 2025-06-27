import { QueryClient } from '@tanstack/solid-query';

export const createQueryClient = () => {
  return new QueryClient({
    defaultOptions: {
      queries: {
        // 5 minutes
        staleTime: 5 * 60 * 1000,
        retry: 2,
        refetchOnWindowFocus: false,
      },
    },
  });
};
