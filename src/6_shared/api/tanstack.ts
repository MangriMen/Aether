import { QueryClient } from '@tanstack/solid-query';

export const createQueryClient = () => {
  return new QueryClient({
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: false,
        retry: 2,
        // 5 minutes
        staleTime: 5 * 60 * 1000,
      },
    },
  });
};
