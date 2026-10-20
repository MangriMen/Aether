import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { settingsCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

import { appSettingsCache, appSettingsQueries } from './appSettingsCache';

export const useAppSettings = () => useQuery(appSettingsQueries.get);

export const useEditAppSettings = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: settingsCommands.editAppSettings,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
    onError: (err) => {
      showError({ title: t('settings.changeError'), err, t });
    },
  }));
};
