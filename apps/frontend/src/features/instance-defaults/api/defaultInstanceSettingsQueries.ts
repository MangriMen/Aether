import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { settingsCommands } from '@/platform/ipc';
import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import {
  defaultInstanceSettingsCache,
  defaultInstanceSettingsQueries,
} from './defaultInstanceSettingsCache';

export const useDefaultInstanceSettings = () =>
  useQuery(defaultInstanceSettingsQueries.get);

export const useEditDefaultInstanceSettings = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: settingsCommands.editDefaultInstanceSettings,
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
