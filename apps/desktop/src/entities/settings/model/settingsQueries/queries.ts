import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '../../../../shared/lib';
import { useTranslation } from '../../../../shared/model';
import { commands } from '../../api';
import { settingsCache, settingsQueries } from './cache';

export const useSettings = () => useQuery(settingsQueries.get);

export const useMaxRam = () => useQuery(settingsQueries.maxRam);

export const useEditSettings = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.edit,
    onSuccess: (data) => {
      settingsCache.set(queryClient, data);
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
