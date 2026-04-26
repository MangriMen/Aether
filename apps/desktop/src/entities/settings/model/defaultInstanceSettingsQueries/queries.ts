import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '../../../../shared/lib';
import { useTranslation } from '../../../../shared/model';
import { commands } from '../../api';
import {
  defaultInstanceSettingsCache,
  defaultInstanceSettingsQueries,
} from './cache';

export const useDefaultInstanceSettings = () =>
  useQuery(defaultInstanceSettingsQueries.get);

export const useEditDefaultInstanceSettings = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.editDefaultInstanceSettings,
    onSuccess: (data) => {
      defaultInstanceSettingsCache.set(queryClient, data);
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
