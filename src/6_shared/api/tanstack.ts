import { QueryClient } from '@tanstack/solid-query';
import { isAetherLauncherError } from '@/shared/model';
import { showToast } from '@/shared/ui';

export const createQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        // 5 minutes
        staleTime: 5 * 60 * 1000,
        retry: 2,
        refetchOnWindowFocus: false,
      },
      mutations: {
        onError: (error) => {
          if (isAetherLauncherError(error)) {
            showToast({
              title: 'Action failed',
              description: error.message,
              variant: 'destructive',
            });
          }
        },
      },
    },
  });
