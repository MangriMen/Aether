import { useMutation, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/features/errors';
import { appSettingsCache } from '@/features/settings';
import { applicationCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

export const useRecreateWindow = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: applicationCommands.recreateWindow,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
    onError: (err) => {
      showError({
        title: t('settings.windowTransparencyDialog.recreateError'),
        err,
        t,
      });
    },
  }));
};
