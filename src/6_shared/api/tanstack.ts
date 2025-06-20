import { QueryClient } from '@tanstack/solid-query';
import { isErrorKind, isLauncherError } from '@/shared/model';
import { showToast } from '@/shared/ui';
import { internalT as t } from '@/1_app/providers';

export const createQueryClient = () => {
  return new QueryClient({
    defaultOptions: {
      queries: {
        // 5 minutes
        staleTime: 5 * 60 * 1000,
        retry: 2,
        refetchOnWindowFocus: false,
      },
      mutations: {
        onError: (error) => {
          if (isLauncherError(error)) {
            const kind = error.code.split('.')[0];
            const description = `errors.${error.code}` as const;

            const translatedTitle = isErrorKind(kind)
              ? t(`errors.${kind}.title`)
              : kind;

            showToast({
              title: translatedTitle,
              description:
                t(description, error.fields ?? undefined) || description,
              variant: 'destructive',
            });
          }
        },
      },
    },
  });
};
