import type { QueryClient } from '@tanstack/solid-query';

import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { commands } from '../../api';
import { settingsKeys } from './queryKeys';

export const useSettings = () =>
  useQuery(() => ({
    queryKey: settingsKeys.get(),
    queryFn: commands.get,
  }));

export const useEditSettings = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.edit,
    onSuccess: (data) => {
      queryClient.setQueryData(settingsKeys.get(), () => data);
    },
    onError: (err) => {
      showError({
        title: t('settings.changeError'),
        err,
        t,
      });
    },
  }));
};

export const MAX_RAM_QUERY = () => ({
  queryKey: settingsKeys.ram(),
  queryFn: commands.getMaxRam,
  staleTime: Infinity,
});

export const useMaxRam = () => useQuery(MAX_RAM_QUERY);

export const prefetchMaxRam = (queryClient: QueryClient) =>
  queryClient.prefetchQuery(MAX_RAM_QUERY());
