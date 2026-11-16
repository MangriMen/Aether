import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { settingsCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

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
    // Read again after a failed edit too, so the forms show what is actually stored.
    onSettled: (data) => {
      if (data) {
        defaultInstanceSettingsCache.set(queryClient, data);
      } else {
        void defaultInstanceSettingsCache.invalidate.get(queryClient);
      }
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
